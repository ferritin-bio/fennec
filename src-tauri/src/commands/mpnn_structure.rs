//! Structure normalisation for the (Ligand|Protein)MPNN featurizer.
//!
//! ferritin's path-based featurizer (`MPNNExecConfig::generate_protein_features`)
//! does not expose which residue each model position belongs to, and its reader
//! has several properties that make a raw structure file a poor input:
//!
//! - it reads *every* model of a multi-model file (NMR ensembles become one
//!   overlapping "structure"),
//! - it keeps every alt-loc atom,
//! - it segments residues on `(auth_seq_id, comp_id, chain)` only, so insertion
//!   codes are invisible (`52 GLY` + `52A GLY` merge into one residue),
//! - it casts residue numbers to `u32` for the positional encoding, so negative
//!   author numbering wraps to ~4e9,
//! - it marks every residue as being on the same chain, so adjacent numbers on
//!   *different* chains read as sequence neighbours,
//! - it keeps amino-acid residues with missing backbone atoms at the origin.
//!
//! So instead of handing it the user's text, we parse the structure here, keep
//! a residue table, and write a minimal normalised mmCIF that ferritin reads.
//! Every residue the featurizer sees is one row of that table, in the same
//! order, and [`verify_feature_alignment`] checks that 1:1 against the
//! featurizer's own output on every call.

use anyhow::{Context, Result, anyhow, bail};
use ferritin_core::AtomCollection;
use ferritin_core::info::amino_acids::{aa1to_int, aa3to1};
use ferritin_core::info::elements::Element;
use serde::Serialize;
use std::collections::HashMap;
use std::fmt::Write as _;

/// Residue names treated as solvent and removed before featurization.
const WATER_NAMES: &[&str] = &["HOH", "WAT", "DOD", "H2O", "D2O", "TIP", "TIP3", "SOL"];
/// Backbone atoms the MPNN featurizer reads; residues missing any are skipped.
const BACKBONE_ATOMS: [&str; 4] = ["N", "CA", "C", "O"];
/// The largest residue-number step kept between consecutive residues of one
/// chain. The model clips relative offsets at ±32, so anything above that is
/// equivalent; clamping keeps staged numbers small.
const MAX_STEP: i64 = 64;
/// Spacing between chains in the staged numbering (well past the ±32 clip).
const CHAIN_GAP: i64 = 1000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StructureFormat {
    Cif,
    Pdb,
}

impl StructureFormat {
    pub(crate) fn parse(format: &str) -> Result<Self> {
        match format.trim().to_ascii_lowercase().as_str() {
            "cif" | "mmcif" | "pdbx" => Ok(Self::Cif),
            "pdb" | "ent" => Ok(Self::Pdb),
            other => bail!("unsupported structure format {other:?}; expected \"cif\" or \"pdb\""),
        }
    }

    /// Best-effort format detection for callers that do not say.
    pub(crate) fn sniff(text: &str) -> Self {
        if text.contains("_atom_site.") {
            Self::Cif
        } else {
            Self::Pdb
        }
    }
}

/// One atom record as read from the input file.
#[derive(Debug, Clone)]
struct ParsedAtom {
    het: bool,
    atom_name: String,
    /// Uppercase element symbol as written in the file ("" if absent).
    element: String,
    comp_id: String,
    auth_asym_id: String,
    label_asym_id: String,
    auth_seq_id: i32,
    label_seq_id: Option<i32>,
    ins_code: String,
    alt_id: String,
    coord: [f32; 3],
    model: i32,
}

/// A polymer residue the model will see, in model (featurizer) order.
#[derive(Debug, Clone)]
pub(crate) struct PolymerResidue {
    /// Author chain id (`auth_asym_id`; the PDB chain column for PDB input).
    pub chain_id: String,
    /// `label_asym_id` for mmCIF input; equal to `chain_id` for PDB input.
    pub label_asym_id: String,
    pub auth_seq_id: i32,
    pub label_seq_id: Option<i32>,
    pub ins_code: String,
    /// Three-letter residue name as written in the file.
    pub comp_id: String,
    /// One-letter code the model is given as the wild type ('X' if non-canonical).
    pub wt: char,
    /// Residue number written to the staged file — what the model's relative
    /// positional encoding sees.
    pub staged_seq_id: i32,
    /// Minimum heavy-atom distance (Å) to any ligand heavy atom, if there are ligands.
    pub ligand_distance: Option<f32>,
}

/// A residue present in the input that the model does not score.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct SkippedResidue {
    pub chain_id: String,
    pub auth_seq_id: i32,
    pub ins_code: String,
    pub comp_id: String,
    pub reason: String,
}

/// The normalised structure handed to the featurizer, plus its residue table.
#[derive(Debug)]
pub(crate) struct PreparedStructure {
    /// Minimal mmCIF that ferritin's featurizer reads.
    pub staged_cif: String,
    /// Polymer residues in exactly the featurizer's order.
    pub residues: Vec<PolymerResidue>,
    pub skipped: Vec<SkippedResidue>,
    /// Ligand heavy atoms visible to LigandMPNN (non-polymer, non-water).
    pub n_ligand_atoms: usize,
}

/// A run of consecutive atoms sharing one residue identity.
struct ResidueGroup {
    atoms: Vec<ParsedAtom>,
    is_amino_acid: bool,
}

impl ResidueGroup {
    fn first(&self) -> &ParsedAtom {
        &self.atoms[0]
    }
}

/// Parse `text`, normalise it and build the staged mmCIF + residue table.
pub(crate) fn prepare_structure(text: &str, format: StructureFormat) -> Result<PreparedStructure> {
    let atoms = match format {
        StructureFormat::Cif => parse_cif_atoms(text)?,
        StructureFormat::Pdb => parse_pdb_atoms(text)?,
    };
    if atoms.is_empty() {
        bail!("structure contains no ATOM/HETATM records");
    }

    // 1. First model only (lowest model number, as ferritin's own Model reader does).
    let first_model = atoms.iter().map(|a| a.model).min().unwrap_or(1);
    // 2. Drop solvent and hydrogens.
    // 3. Alt-locs: per residue position keep blank alt ids plus the first alt id seen.
    let mut first_alt: HashMap<(String, i32, String), String> = HashMap::new();
    let atoms: Vec<ParsedAtom> = atoms
        .into_iter()
        .filter(|a| a.model == first_model)
        .filter(|a| !WATER_NAMES.contains(&a.comp_id.to_ascii_uppercase().as_str()))
        .filter(|a| !is_hydrogen(a))
        .filter(|a| {
            if a.alt_id.is_empty() {
                return true;
            }
            let key = (a.auth_asym_id.clone(), a.auth_seq_id, a.ins_code.clone());
            let kept = first_alt.entry(key).or_insert_with(|| a.alt_id.clone());
            *kept == a.alt_id
        })
        .collect();

    // 4. Group consecutive atoms into residues by full identity (incl. insertion code).
    let mut groups: Vec<ResidueGroup> = Vec::new();
    for atom in atoms {
        let same = groups.last().is_some_and(|g| {
            let f = g.first();
            f.auth_asym_id == atom.auth_asym_id
                && f.label_asym_id == atom.label_asym_id
                && f.auth_seq_id == atom.auth_seq_id
                && f.ins_code == atom.ins_code
                && f.comp_id == atom.comp_id
        });
        if same {
            groups.last_mut().unwrap().atoms.push(atom);
        } else {
            groups.push(ResidueGroup {
                atoms: vec![atom],
                is_amino_acid: false,
            });
        }
    }

    // 5. Classify with ferritin's own amino-acid table, so "polymer residue"
    //    means exactly what the featurizer's `iter_residues_aminoacid` means.
    let flags = classify_amino_acids(&groups);
    for (g, flag) in groups.iter_mut().zip(flags) {
        g.is_amino_acid = flag;
    }

    // 6. Skip amino acids without a full backbone; fix up / drop atoms whose
    //    element cannot be determined.
    let mut skipped = Vec::new();
    let mut kept: Vec<ResidueGroup> = Vec::with_capacity(groups.len());
    for mut g in groups {
        if g.is_amino_acid {
            let missing: Vec<&str> = BACKBONE_ATOMS
                .iter()
                .copied()
                .filter(|name| !g.atoms.iter().any(|a| a.atom_name == *name))
                .collect();
            if !missing.is_empty() {
                let f = g.first();
                skipped.push(SkippedResidue {
                    chain_id: f.auth_asym_id.clone(),
                    auth_seq_id: f.auth_seq_id,
                    ins_code: f.ins_code.clone(),
                    comp_id: f.comp_id.clone(),
                    reason: format!("missing backbone atom(s): {}", missing.join(", ")),
                });
                continue;
            }
            for a in &mut g.atoms {
                // Element is irrelevant for polymer atoms (atom37 is looked up by
                // name), but must still be a valid symbol in the staged file.
                a.element = resolve_element(a).unwrap_or_else(|| "C".to_string());
            }
        } else {
            g.atoms.retain_mut(|a| match resolve_element(a) {
                Some(el) => {
                    a.element = el;
                    true
                }
                None => false,
            });
            if g.atoms.is_empty() {
                continue;
            }
        }
        kept.push(g);
    }

    // 7. Staged numbering. Within a chain, consecutive polymer residues step by
    //    their author-number difference (clamped to 1..=MAX_STEP), so gaps are
    //    preserved, insertion codes become distinct consecutive positions and
    //    negative numbers cannot wrap. Chains are separated by CHAIN_GAP.
    //    Non-polymer residues get unique numbers after all polymer residues.
    let mut chain_index: HashMap<String, usize> = HashMap::new();
    let mut chain_state: HashMap<String, (i32, i64)> = HashMap::new();
    let mut high_water: i64 = 1 - CHAIN_GAP;
    let mut staged_numbers: Vec<i64> = vec![0; kept.len()];
    for (i, g) in kept.iter().enumerate() {
        let f = g.first();
        let n = chain_index.len();
        chain_index.entry(f.auth_asym_id.clone()).or_insert(n);
        if !g.is_amino_acid {
            continue;
        }
        let staged = match chain_state.get(&f.auth_asym_id) {
            Some(&(last_auth, last_staged)) => {
                last_staged + (f.auth_seq_id as i64 - last_auth as i64).clamp(1, MAX_STEP)
            }
            None => high_water + CHAIN_GAP,
        };
        chain_state.insert(f.auth_asym_id.clone(), (f.auth_seq_id, staged));
        high_water = high_water.max(staged);
        staged_numbers[i] = staged;
    }
    let mut next_ligand = high_water.max(0) + CHAIN_GAP;
    for (i, g) in kept.iter().enumerate() {
        if !g.is_amino_acid {
            staged_numbers[i] = next_ligand;
            next_ligand += 1;
        }
    }
    if next_ligand > i32::MAX as i64 / 2 {
        bail!("structure too large to stage for MPNN");
    }

    // 8. Ligand atoms (everything kept that is not a polymer residue).
    let ligand_coords: Vec<[f32; 3]> = kept
        .iter()
        .filter(|g| !g.is_amino_acid)
        .flat_map(|g| g.atoms.iter().map(|a| a.coord))
        .collect();

    // 9. Residue table + staged mmCIF.
    let mut residues = Vec::new();
    let mut staged_cif =
        String::with_capacity(kept.iter().map(|g| g.atoms.len()).sum::<usize>() * 90);
    staged_cif.push_str(STAGED_HEADER);
    let mut atom_serial = 0usize;
    for (g, &staged) in kept.iter().zip(&staged_numbers) {
        let f = g.first();
        let chain_token = format!("C{}", chain_index[&f.auth_asym_id]);
        if g.is_amino_acid {
            let ligand_distance = min_distance(&g.atoms, &ligand_coords);
            residues.push(PolymerResidue {
                chain_id: f.auth_asym_id.clone(),
                label_asym_id: f.label_asym_id.clone(),
                auth_seq_id: f.auth_seq_id,
                label_seq_id: f.label_seq_id,
                ins_code: f.ins_code.clone(),
                comp_id: f.comp_id.clone(),
                wt: aa3to1(&f.comp_id),
                staged_seq_id: staged as i32,
                ligand_distance,
            });
        }
        for a in &g.atoms {
            atom_serial += 1;
            let _ = writeln!(
                staged_cif,
                "{group} {atom_serial} {el} {name} . {comp} {chain} . ? {x:.3} {y:.3} {z:.3} 1.00 0.00 {seq} {comp} {chain} {name} 1",
                group = if g.is_amino_acid { "ATOM" } else { "HETATM" },
                el = a.element,
                name = cif_token(&a.atom_name),
                comp = cif_token(&a.comp_id),
                chain = chain_token,
                seq = staged,
                x = a.coord[0],
                y = a.coord[1],
                z = a.coord[2],
            );
        }
    }
    staged_cif.push_str("#\n");

    Ok(PreparedStructure {
        staged_cif,
        residues,
        skipped,
        n_ligand_atoms: ligand_coords.len(),
    })
}

const STAGED_HEADER: &str = "data_fennec_mpnn
#
loop_
_atom_site.group_PDB
_atom_site.id
_atom_site.type_symbol
_atom_site.label_atom_id
_atom_site.label_alt_id
_atom_site.label_comp_id
_atom_site.label_asym_id
_atom_site.label_seq_id
_atom_site.pdbx_PDB_ins_code
_atom_site.Cartn_x
_atom_site.Cartn_y
_atom_site.Cartn_z
_atom_site.occupancy
_atom_site.B_iso_or_equiv
_atom_site.auth_seq_id
_atom_site.auth_comp_id
_atom_site.auth_asym_id
_atom_site.auth_atom_id
_atom_site.pdbx_PDB_model_num
";

/// Check the featurizer's residue indices and sequence against the residue
/// table, position by position. Staged numbers are globally unique, so equal
/// `r_idx` vectors pin every model position to exactly one table row.
pub(crate) fn verify_feature_alignment(
    residues: &[PolymerResidue],
    r_idx: &[u32],
    seq: &[u32],
) -> Result<()> {
    if r_idx.len() != residues.len() || seq.len() != residues.len() {
        bail!(
            "featurizer produced {} residues (sequence {}), residue table has {}",
            r_idx.len(),
            seq.len(),
            residues.len()
        );
    }
    for (i, ((res, &ri), &s)) in residues.iter().zip(r_idx).zip(seq).enumerate() {
        if ri as i64 != res.staged_seq_id as i64 || s != aa1to_int(res.wt) {
            bail!(
                "featurizer position {i} does not match residue {}:{}{} {} (r_idx {ri} vs {}, aa {s} vs {})",
                res.chain_id,
                res.auth_seq_id,
                res.ins_code,
                res.comp_id,
                res.staged_seq_id,
                aa1to_int(res.wt)
            );
        }
    }
    Ok(())
}

/// Numerically stable `softmax(values / temperature)`.
///
/// Because log-softmax only shifts each row by a constant, applying this to
/// log-probabilities gives exactly the same result as applying it to raw logits.
pub(crate) fn softmax_with_temperature(values: &[f32], temperature: f32) -> Vec<f32> {
    let t = if temperature.is_finite() && temperature > 0.0 {
        temperature.max(1e-6)
    } else {
        1.0
    };
    let max = values
        .iter()
        .map(|v| v / t)
        .fold(f32::NEG_INFINITY, f32::max);
    let exps: Vec<f32> = values.iter().map(|v| (v / t - max).exp()).collect();
    let sum: f32 = exps.iter().sum();
    exps.into_iter().map(|e| e / sum).collect()
}

// ---------------------------------------------------------------------------
// helpers

fn is_hydrogen(a: &ParsedAtom) -> bool {
    match a.element.as_str() {
        "H" | "D" => true,
        "" => resolve_element(a).as_deref() == Some("H"),
        _ => false,
    }
}

/// A valid element symbol for `a`, from the element column or inferred from the atom name.
fn resolve_element(a: &ParsedAtom) -> Option<String> {
    if !a.element.is_empty() && Element::from_symbol(&a.element).is_some() {
        return Some(a.element.to_ascii_uppercase());
    }
    let name: String = a
        .atom_name
        .trim_start_matches(|c: char| c.is_ascii_digit())
        .chars()
        .take_while(|c| c.is_ascii_alphabetic())
        .collect::<String>()
        .to_ascii_uppercase();
    if name.is_empty() {
        return None;
    }
    // Single-atom ions are usually named after their element (ZN, FE, MG...).
    if a.het
        && name.len() >= 2
        && a.comp_id.eq_ignore_ascii_case(&name[..2])
        && Element::from_symbol(&name[..2]).is_some()
    {
        return Some(name[..2].to_string());
    }
    Element::from_symbol(&name[..1]).map(|_| name[..1].to_string())
}

/// Run each group's residue name through ferritin's amino-acid classifier.
fn classify_amino_acids(groups: &[ResidueGroup]) -> Vec<bool> {
    let n = groups.len();
    if n == 0 {
        return Vec::new();
    }
    // One placeholder atom per group, with distinct residue ids so ferritin
    // segments exactly one residue per group.
    let ac = AtomCollection::new(
        n,
        vec![[0.0; 3]; n],
        (0..n as i32).collect(),
        groups.iter().map(|g| g.first().comp_id.clone()).collect(),
        vec![false; n],
        vec![Element::C; n],
        vec!["CA".to_string(); n],
        vec!["A".to_string(); n],
        None,
    );
    let flags: Vec<bool> = ac.iter_residues().map(|r| r.is_amino_acid()).collect();
    debug_assert_eq!(flags.len(), n);
    flags
}

fn min_distance(atoms: &[ParsedAtom], ligand: &[[f32; 3]]) -> Option<f32> {
    if ligand.is_empty() {
        return None;
    }
    let mut best = f32::INFINITY;
    for a in atoms {
        for l in ligand {
            let d2 = (a.coord[0] - l[0]).powi(2)
                + (a.coord[1] - l[1]).powi(2)
                + (a.coord[2] - l[2]).powi(2);
            best = best.min(d2);
        }
    }
    Some(best.sqrt())
}

/// A token that ferritin's CIF reader will read back verbatim.
fn cif_token(value: &str) -> String {
    let cleaned: String = value
        .chars()
        .map(|c| {
            if c.is_whitespace() || c == '\'' || c == '"' {
                '_'
            } else {
                c
            }
        })
        .collect();
    match cleaned.as_str() {
        "" | "." | "?" => "X".to_string(),
        s if s.starts_with(['_', '#', '$', ';', '[', ']']) => format!("X{s}"),
        s => s.to_string(),
    }
}

// ---------------------------------------------------------------------------
// PDB reader

/// Fixed-column field of a PDB line, tolerant of short lines.
fn col(line: &str, start: usize, end: usize) -> &str {
    let end = end.min(line.len());
    if start >= end {
        return "";
    }
    line.get(start..end).unwrap_or("")
}

fn parse_pdb_atoms(text: &str) -> Result<Vec<ParsedAtom>> {
    let mut atoms = Vec::new();
    let mut model = 1;
    let mut model_counter = 0;
    for (line_no, line) in text.lines().enumerate() {
        if line.starts_with("MODEL") {
            model_counter += 1;
            model = col(line, 10, 14).trim().parse().unwrap_or(model_counter);
            continue;
        }
        let het = line.starts_with("HETATM");
        if !(het || line.starts_with("ATOM")) {
            continue;
        }
        let ctx = || format!("PDB line {}: {line:?}", line_no + 1);
        let parse_f = |s: &str| s.trim().parse::<f32>();
        let coord = [
            parse_f(col(line, 30, 38)).with_context(ctx)?,
            parse_f(col(line, 38, 46)).with_context(ctx)?,
            parse_f(col(line, 46, 54)).with_context(ctx)?,
        ];
        let auth_seq_id: i32 = col(line, 22, 26)
            .trim()
            .parse()
            .with_context(|| format!("invalid residue number; {}", ctx()))?;
        let chain = col(line, 21, 22).trim().to_string();
        atoms.push(ParsedAtom {
            het,
            atom_name: col(line, 12, 16).trim().to_string(),
            element: col(line, 76, 78).trim().to_ascii_uppercase(),
            comp_id: col(line, 17, 21).trim().to_string(),
            label_asym_id: chain.clone(),
            auth_asym_id: chain,
            auth_seq_id,
            label_seq_id: None,
            ins_code: col(line, 26, 27).trim().to_string(),
            alt_id: col(line, 16, 17).trim().to_string(),
            coord,
            model,
        });
    }
    Ok(atoms)
}

// ---------------------------------------------------------------------------
// mmCIF reader (atom_site loop only)

#[derive(Debug, PartialEq)]
struct CifToken<'a> {
    value: std::borrow::Cow<'a, str>,
    quoted: bool,
}

fn tokenize_cif(text: &str) -> Vec<CifToken<'_>> {
    let mut tokens = Vec::new();
    let mut lines = text.lines();
    while let Some(line) = lines.next() {
        if let Some(rest) = line.strip_prefix(';') {
            // Semicolon-delimited text field.
            let mut buf = rest.to_string();
            for l in lines.by_ref() {
                if l.starts_with(';') {
                    break;
                }
                buf.push('\n');
                buf.push_str(l);
            }
            tokens.push(CifToken {
                value: buf.into(),
                quoted: true,
            });
            continue;
        }
        let bytes = line.as_bytes();
        let len = bytes.len();
        let mut i = 0;
        while i < len {
            let c = bytes[i];
            if c.is_ascii_whitespace() {
                i += 1;
            } else if c == b'#' {
                break;
            } else if c == b'\'' || c == b'"' {
                // A quote only closes when followed by whitespace or end of line.
                let mut j = i + 1;
                while j < len
                    && !(bytes[j] == c && (j + 1 == len || bytes[j + 1].is_ascii_whitespace()))
                {
                    j += 1;
                }
                tokens.push(CifToken {
                    value: line[i + 1..j.min(len)].into(),
                    quoted: true,
                });
                i = j + 1;
            } else {
                let start = i;
                while i < len && !bytes[i].is_ascii_whitespace() {
                    i += 1;
                }
                tokens.push(CifToken {
                    value: line[start..i].into(),
                    quoted: false,
                });
            }
        }
    }
    tokens
}

fn is_reserved(tok: &CifToken) -> bool {
    if tok.quoted {
        return false;
    }
    let v = tok.value.to_ascii_lowercase();
    v.starts_with('_')
        || v == "loop_"
        || v.starts_with("data_")
        || v.starts_with("save_")
        || v == "global_"
        || v == "stop_"
}

fn parse_cif_atoms(text: &str) -> Result<Vec<ParsedAtom>> {
    let tokens = tokenize_cif(text);
    let mut i = 0;
    let (columns, values) = loop {
        if i >= tokens.len() {
            bail!("no _atom_site loop found in mmCIF text");
        }
        let is_loop = !tokens[i].quoted && tokens[i].value.eq_ignore_ascii_case("loop_");
        let next_is_atom_site = tokens
            .get(i + 1)
            .is_some_and(|t| !t.quoted && t.value.to_ascii_lowercase().starts_with("_atom_site."));
        if !(is_loop && next_is_atom_site) {
            i += 1;
            continue;
        }
        i += 1;
        let mut columns = Vec::new();
        while let Some(t) = tokens.get(i) {
            if t.quoted || !t.value.to_ascii_lowercase().starts_with("_atom_site.") {
                break;
            }
            columns.push(t.value["_atom_site.".len()..].to_string());
            i += 1;
        }
        let start = i;
        while i < tokens.len() && !is_reserved(&tokens[i]) {
            i += 1;
        }
        break (columns, &tokens[start..i]);
    };
    let ncol = columns.len();
    if values.len() % ncol != 0 {
        bail!(
            "malformed _atom_site loop: {} values for {} columns",
            values.len(),
            ncol
        );
    }

    let idx = |name: &str| columns.iter().position(|c| c == name);
    let require = |names: &[&str]| -> Result<usize> {
        names
            .iter()
            .find_map(|n| idx(n))
            .ok_or_else(|| anyhow!("_atom_site is missing column {}", names.join(" / ")))
    };
    let x_col = require(&["Cartn_x"])?;
    let y_col = require(&["Cartn_y"])?;
    let z_col = require(&["Cartn_z"])?;
    let atom_col = require(&["auth_atom_id", "label_atom_id"])?;
    let comp_col = require(&["auth_comp_id", "label_comp_id"])?;
    let chain_col = require(&["auth_asym_id", "label_asym_id"])?;
    let auth_seq_col = idx("auth_seq_id");
    let label_seq_col = idx("label_seq_id");
    if auth_seq_col.is_none() && label_seq_col.is_none() {
        bail!("_atom_site is missing column auth_seq_id / label_seq_id");
    }
    let label_asym_col = idx("label_asym_id");
    let group_col = idx("group_PDB");
    let element_col = idx("type_symbol");
    let ins_col = idx("pdbx_PDB_ins_code");
    let alt_col = idx("label_alt_id");
    let model_col = idx("pdbx_PDB_model_num");

    let mut atoms = Vec::with_capacity(values.len() / ncol);
    for (row_no, row) in values.chunks(ncol).enumerate() {
        // Unquoted '.' and '?' are CIF null values.
        let get = |c: Option<usize>| -> &str {
            match c.map(|c| &row[c]) {
                Some(t) if !t.quoted && (t.value == "." || t.value == "?") => "",
                Some(t) => t.value.as_ref(),
                None => "",
            }
        };
        let ctx = || format!("_atom_site row {}", row_no + 1);
        let parse_f = |c: usize| get(Some(c)).parse::<f32>().with_context(ctx);
        let label_seq_id = get(label_seq_col).parse::<i32>().ok();
        let auth_seq_raw = get(auth_seq_col);
        let auth_seq_id = if auth_seq_raw.is_empty() {
            label_seq_id.ok_or_else(|| anyhow!("{}: no residue number", ctx()))?
        } else {
            auth_seq_raw
                .parse::<i32>()
                .with_context(|| format!("{}: invalid auth_seq_id {auth_seq_raw:?}", ctx()))?
        };
        let auth_asym_id = get(Some(chain_col)).to_string();
        let label_asym_id = match get(label_asym_col) {
            "" => auth_asym_id.clone(),
            s => s.to_string(),
        };
        atoms.push(ParsedAtom {
            het: get(group_col) == "HETATM",
            atom_name: get(Some(atom_col)).to_string(),
            element: get(element_col).to_ascii_uppercase(),
            comp_id: get(Some(comp_col)).to_string(),
            auth_asym_id,
            label_asym_id,
            auth_seq_id,
            label_seq_id,
            ins_code: get(ins_col).to_string(),
            alt_id: get(alt_col).to_string(),
            coord: [parse_f(x_col)?, parse_f(y_col)?, parse_f(z_col)?],
            model: get(model_col).parse().unwrap_or(1),
        });
    }
    Ok(atoms)
}

#[cfg(test)]
mod tests {
    use super::*;
    use candle_core::Device;
    use ferritin_plms::ligandmpnn::configs::{MPNNExecConfig, RunConfig};
    use std::io::Write;

    const CIF_101M: &str = include_str!("../../testdata/structures/101m.cif");
    const PDB_1BC8: &str = include_str!("../../testdata/structures/1BC8.pdb");

    /// Run ferritin's real featurizer (no weights needed) on the staged file.
    pub(crate) fn featurize(prepared: &PreparedStructure) -> (Vec<u32>, Vec<u32>) {
        let mut f = tempfile::Builder::new().suffix(".cif").tempfile().unwrap();
        f.write_all(prepared.staged_cif.as_bytes()).unwrap();
        let exec = MPNNExecConfig::new(
            Device::Cpu,
            f.path().to_string_lossy().into_owned(),
            RunConfig {
                model_type: None,
                seed: None,
                temperature: None,
                verbose: None,
                save_stats: None,
                batch_size: None,
                number_of_batches: None,
                file_ending: None,
                zero_indexed: None,
                homo_oligomer: None,
                fasta_seq_separation: None,
            },
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
        let features = exec.generate_protein_features().unwrap();
        let r_idx = features
            .get_residue_index()
            .flatten_all()
            .unwrap()
            .to_vec1::<u32>()
            .unwrap();
        let seq = features
            .get_sequence()
            .flatten_all()
            .unwrap()
            .to_vec1::<u32>()
            .unwrap();
        (r_idx, seq)
    }

    fn entity_seq_101m() -> String {
        let start = CIF_101M
            .find("_entity_poly.pdbx_seq_one_letter_code_can")
            .unwrap();
        let block = &CIF_101M[start..];
        let open = block.find("\n;").unwrap() + 2;
        let close = block[open..].find("\n;").unwrap() + open;
        block[open..close].split_whitespace().collect()
    }

    #[test]
    fn test_101m_residue_table_matches_entity_and_featurizer() {
        let p = prepare_structure(CIF_101M, StructureFormat::Cif).unwrap();
        assert_eq!(p.residues.len(), 154);
        assert!(p.skipped.is_empty(), "{:?}", p.skipped);
        assert!(
            p.residues
                .iter()
                .all(|r| r.chain_id == "A" && r.ins_code.is_empty())
        );
        // Author numbering in 101m starts at 0 (label_seq_id starts at 1).
        assert_eq!(p.residues[0].auth_seq_id, 0);
        assert_eq!(p.residues[0].label_seq_id, Some(1));
        assert_eq!(p.residues[153].auth_seq_id, 153);
        let seq: String = p.residues.iter().map(|r| r.wt).collect();
        assert_eq!(seq, entity_seq_101m());
        // HEM (43) + NBN (6) + SO4 (5); 138 waters excluded.
        assert_eq!(p.n_ligand_atoms, 54);
        // Staged numbers preserve offsets (contiguous chain, base 1).
        assert!(
            p.residues
                .iter()
                .all(|r| r.staged_seq_id == r.auth_seq_id + 1)
        );
        // The heme-ligating proximal His93 sits right at the heme iron.
        let his93 = p.residues.iter().find(|r| r.auth_seq_id == 93).unwrap();
        assert_eq!(his93.wt, 'H');
        assert!(his93.ligand_distance.unwrap() < 3.0);

        let (r_idx, s) = featurize(&p);
        verify_feature_alignment(&p.residues, &r_idx, &s).unwrap();
    }

    #[test]
    fn test_1bc8_pdb_matches_ferritin_reader() {
        let p = prepare_structure(PDB_1BC8, StructureFormat::Pdb).unwrap();
        // Cross-check against ferritin reading the raw PDB directly.
        let ac = ferritin_core::load_structure_from_string(PDB_1BC8, "pdb").unwrap();
        let direct: Vec<(String, i32, String)> = ac
            .iter_residues_aminoacid()
            .map(|r| {
                (
                    r.chain_id().to_string(),
                    r.residue_id(),
                    r.residue_name().to_string(),
                )
            })
            .collect();
        let ours: Vec<(String, i32, String)> = p
            .residues
            .iter()
            .map(|r| (r.chain_id.clone(), r.auth_seq_id, r.comp_id.clone()))
            .collect();
        assert_eq!(ours, direct);
        // ferritin documents 406 ligand atoms for 1BC8: 18-mer DNA duplex + 2 Zn.
        assert_eq!(p.n_ligand_atoms, 406);
        let (r_idx, s) = featurize(&p);
        verify_feature_alignment(&p.residues, &r_idx, &s).unwrap();
    }

    #[allow(clippy::too_many_arguments)] // mirrors the PDB ATOM record columns
    fn pdb_line(
        het: bool,
        serial: usize,
        name: &str,
        alt: &str,
        res: &str,
        chain: &str,
        seq: i32,
        icode: &str,
        xyz: [f32; 3],
        el: &str,
    ) -> String {
        format!(
            "{:<6}{:>5} {:<4}{:1}{:>3} {:1}{:>4}{:1}   {:>8.3}{:>8.3}{:>8.3}  1.00  0.00          {:>2}",
            if het { "HETATM" } else { "ATOM" },
            serial,
            name,
            alt,
            res,
            chain,
            seq,
            icode,
            xyz[0],
            xyz[1],
            xyz[2],
            el
        )
    }

    fn residue_lines(
        out: &mut Vec<String>,
        res: &str,
        chain: &str,
        seq: i32,
        icode: &str,
        x: f32,
        alt: &str,
    ) {
        for (k, name) in ["N", "CA", "C", "O"].iter().enumerate() {
            let n = out.len() + 1;
            out.push(pdb_line(
                false,
                n,
                name,
                alt,
                res,
                chain,
                seq,
                icode,
                [x, k as f32 * 1.4, 0.0],
                &name[..1],
            ));
        }
    }

    #[test]
    fn test_insertion_codes_negative_numbers_altlocs_models() {
        let mut l = vec!["MODEL        1".to_string()];
        residue_lines(&mut l, "MET", "H", -2, "", 0.0, "");
        residue_lines(&mut l, "GLY", "H", -1, "", 3.8, "");
        residue_lines(&mut l, "GLY", "H", 52, "", 7.6, "");
        residue_lines(&mut l, "GLY", "H", 52, "A", 11.4, ""); // same name, insertion code
        residue_lines(&mut l, "SER", "H", 53, "", 15.2, "A"); // alt A
        residue_lines(&mut l, "SER", "H", 53, "", 15.3, "B"); // alt B, dropped
        // Residue missing O.
        l.push(pdb_line(
            false,
            99,
            "N",
            "",
            "ALA",
            "H",
            54,
            "",
            [19.0, 0.0, 0.0],
            "N",
        ));
        l.push(pdb_line(
            false,
            99,
            "CA",
            "",
            "ALA",
            "H",
            54,
            "",
            [19.0, 1.4, 0.0],
            "C",
        ));
        l.push(pdb_line(
            false,
            99,
            "H",
            "",
            "ALA",
            "H",
            54,
            "",
            [19.0, 2.0, 0.0],
            "H",
        ));
        residue_lines(&mut l, "LYS", "L", 1, "", 0.0, "");
        l.push(pdb_line(
            true,
            99,
            "O",
            "",
            "HOH",
            "L",
            200,
            "",
            [1.0, 1.0, 1.0],
            "O",
        ));
        l.push(pdb_line(
            true,
            99,
            "ZN",
            "",
            "ZN",
            "L",
            300,
            "",
            [2.0, 2.0, 2.0],
            "",
        ));
        l.push("ENDMDL".to_string());
        l.push("MODEL        2".to_string());
        residue_lines(&mut l, "TRP", "H", 1, "", 0.0, "");
        l.push("ENDMDL".to_string());
        let text = l.join("\n");

        let p = prepare_structure(&text, StructureFormat::Pdb).unwrap();
        let ids: Vec<(String, i32, String, char)> = p
            .residues
            .iter()
            .map(|r| (r.chain_id.clone(), r.auth_seq_id, r.ins_code.clone(), r.wt))
            .collect();
        assert_eq!(
            ids,
            vec![
                ("H".into(), -2, "".into(), 'M'),
                ("H".into(), -1, "".into(), 'G'),
                ("H".into(), 52, "".into(), 'G'),
                ("H".into(), 52, "A".into(), 'G'),
                ("H".into(), 53, "".into(), 'S'),
                ("L".into(), 1, "".into(), 'K'),
            ]
        );
        assert_eq!(p.skipped.len(), 1);
        assert_eq!(p.skipped[0].auth_seq_id, 54);
        // Zn is ligand context (inferred element), water is not.
        assert_eq!(p.n_ligand_atoms, 1);
        // Staged numbering: gaps kept (clamped), insertion is +1, chains far apart.
        let staged: Vec<i32> = p.residues.iter().map(|r| r.staged_seq_id).collect();
        // -1 -> 52 is a 53-residue gap (under the clamp); 52 -> 52A -> 53 step by one.
        assert_eq!(&staged[..5], &[1, 2, 55, 56, 57]);
        assert!(staged[5] - staged[4] >= CHAIN_GAP as i32);

        let (r_idx, s) = featurize(&p);
        verify_feature_alignment(&p.residues, &r_idx, &s).unwrap();
    }

    #[test]
    fn test_cif_tokenizer_quotes_and_text_fields() {
        let toks = tokenize_cif("a 'b c' \"it's\" d# #comment\n;text\nmore\n;\n'x'y' z");
        let vals: Vec<&str> = toks.iter().map(|t| t.value.as_ref()).collect();
        assert_eq!(
            vals,
            vec!["a", "b c", "it's", "d#", "text\nmore", "x'y", "z"]
        );
    }

    #[test]
    fn test_format_parsing() {
        assert_eq!(StructureFormat::parse("CIF").unwrap(), StructureFormat::Cif);
        assert_eq!(StructureFormat::parse("pdb").unwrap(), StructureFormat::Pdb);
        assert!(StructureFormat::parse("sdf").is_err());
        assert_eq!(StructureFormat::sniff(CIF_101M), StructureFormat::Cif);
        assert_eq!(StructureFormat::sniff(PDB_1BC8), StructureFormat::Pdb);
    }

    #[test]
    fn test_softmax_with_temperature() {
        let logp = [-0.1f32, -2.5, -4.0];
        let p1 = softmax_with_temperature(&logp, 1.0);
        assert!((p1.iter().sum::<f32>() - 1.0).abs() < 1e-6);
        // Shift invariance: raw logits and log-probs give the same distribution.
        let shifted: Vec<f32> = logp.iter().map(|v| v + 7.3).collect();
        let p2 = softmax_with_temperature(&shifted, 0.5);
        let p3 = softmax_with_temperature(&logp, 0.5);
        for (a, b) in p2.iter().zip(&p3) {
            assert!((a - b).abs() < 1e-6);
        }
        // Lower temperature sharpens.
        assert!(p3[0] > p1[0]);
        // Invalid temperature falls back to 1.
        assert_eq!(softmax_with_temperature(&logp, 0.0), p1);
    }
}
