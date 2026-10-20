//! ProteinMPNN / LigandMPNN design logits.
//!
//! [`mpnn_design_logits`] scores a whole structure in one forward pass and
//! returns an `L x 21` matrix of per-residue amino-acid log-probabilities,
//! with each row keyed by the residue's author chain / number / insertion code
//! so the frontend can match Mol* picks. Temperature is applied on the
//! frontend (`softmax(logits / T)`).
//!
//! Loaded models are cached in [`MpnnState`]; inference runs on a blocking
//! thread so the UI stays responsive.
pub use super::mpnn_structure::SkippedResidue;
use super::mpnn_structure::{
    PolymerResidue, StructureFormat, prepare_structure, softmax_with_temperature,
    verify_feature_alignment,
};
use anyhow::{Context, Result, anyhow, bail};
use candle_core::Device;
use ferritin_core::info::amino_acids::ALPHABET;
use ferritin_plms::ligandmpnn::configs::{MPNNExecConfig, RunConfig};
use ferritin_plms::types::PseudoProbability;
use ferritin_plms::{ProteinMPNNModels, ProteinMPNNRunner, device};
use std::io::Write;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Instant;

/// ProteinMPNN, `proteinmpnn_v_48_020` (backbone only).
pub const MODEL_PROTEIN_MPNN: &str = "protein_mpnn_v48_020";
/// LigandMPNN, `ligandmpnn_v_32_020_25` (backbone + 25 nearest ligand atoms per residue).
pub const MODEL_LIGAND_MPNN: &str = "ligand_mpnn_v32_020_25";

/// One residue's row of the design matrix.
#[derive(serde::Serialize, Clone, Debug)]
pub struct ResidueLogits {
    /// 0-based position in the model's residue order (== index in `DesignLogits.residues`).
    pub index: usize,
    /// AUTHOR chain id: `auth_asym_id` for mmCIF, the chain column for PDB.
    /// Matches Mol* `auth_asym_id`.
    pub chain_id: String,
    /// Author residue number (`auth_seq_id` / PDB resSeq). Matches Mol* `auth_seq_id`.
    pub auth_seq_id: i32,
    /// Insertion code (`pdbx_PDB_ins_code` / PDB iCode), "" if none.
    pub ins_code: String,
    /// One-letter wild type as fed to the model; "X" for non-canonical residues (e.g. MSE).
    pub wt: String,
    /// Log-probabilities (log-softmax of the raw logits) over `alphabet`, length == alphabet.len().
    /// `softmax(logits / T)` is identical whether applied to these or to the raw logits.
    pub logits: Vec<f32>,
    /// `label_asym_id` for mmCIF input; equal to `chain_id` for PDB input.
    pub label_asym_id: String,
    /// `label_seq_id` for mmCIF input; null for PDB input.
    pub label_seq_id: Option<i32>,
    /// Three-letter residue name from the file (e.g. "HIS", "MSE").
    pub comp_id: String,
    /// Minimum heavy-atom distance (Å) from this residue to any ligand heavy atom
    /// (any non-polymer, non-water residue — the atoms LigandMPNN sees as context;
    /// nucleic acids and ions count). Null when the structure has no ligand atoms.
    pub ligand_distance: Option<f32>,
}

/// The full design matrix for one structure.
#[derive(serde::Serialize, Clone, Debug)]
pub struct DesignLogits {
    /// Model id, one of [`mpnn_available_models`].
    pub model: String,
    /// One-letter codes in logit order: the 20 canonical amino acids then "X".
    pub alphabet: Vec<String>,
    pub residues: Vec<ResidueLogits>,
    /// What `ResidueLogits.logits` holds; always "log_probs".
    pub logits_kind: String,
    /// Whether the model conditioned on ligand atoms (LigandMPNN with >= 1 ligand atom).
    pub uses_ligand_context: bool,
    /// Ligand heavy atoms found (non-polymer, non-water, first model, first alt-loc).
    pub n_ligand_atoms: usize,
    /// Amino-acid residues present in the input but not scored.
    pub skipped_residues: Vec<SkippedResidue>,
    /// Compute device the forward pass actually ran on: "metal", "cuda" or "cpu".
    /// ProteinMPNN uses the GPU when present; LigandMPNN always reports "cpu"
    /// (see `MpnnModel::device`).
    pub device: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MpnnModel {
    Protein,
    Ligand,
}

impl MpnnModel {
    const ALL: [MpnnModel; 2] = [MpnnModel::Protein, MpnnModel::Ligand];

    fn id(self) -> &'static str {
        match self {
            Self::Protein => MODEL_PROTEIN_MPNN,
            Self::Ligand => MODEL_LIGAND_MPNN,
        }
    }

    fn weights(self) -> ProteinMPNNModels {
        match self {
            Self::Protein => ProteinMPNNModels::V48_020,
            Self::Ligand => ProteinMPNNModels::LigandV32_020_25,
        }
    }

    /// Device to run this model on.
    ///
    /// ProteinMPNN uses the GPU when available (Metal output matches CPU to
    /// ~2e-5 in log-prob). LigandMPNN is pinned to the CPU: at ferritin rev
    /// 887f6eb its ligand-context path on Metal returns wrong values
    /// (max |delta log p| ~ 3 vs CPU) and intermittently NaN, while the CPU path
    /// is the one ferritin verifies against the Python reference.
    fn device(self) -> Result<Device> {
        match self {
            Self::Protein => device(false).map_err(|e| anyhow!("no compute device: {e}")),
            Self::Ligand => Ok(Device::Cpu),
        }
    }

    /// Accepts the canonical ids, the short aliases and ferritin's registry ids.
    fn from_request(name: &str) -> Result<Self> {
        let n = name.trim().to_ascii_lowercase().replace('-', "_");
        match n.as_str() {
            "protein_mpnn" | "proteinmpnn" | MODEL_PROTEIN_MPNN | "proteinmpnn_v48_020" => {
                Ok(Self::Protein)
            }
            "ligand_mpnn" | "ligandmpnn" | MODEL_LIGAND_MPNN | "ligandmpnn_v32_020_25" => {
                Ok(Self::Ligand)
            }
            _ => bail!(
                "unknown MPNN model {name:?}; expected one of: protein_mpnn, ligand_mpnn, {}, {}",
                MODEL_PROTEIN_MPNN,
                MODEL_LIGAND_MPNN
            ),
        }
    }
}

/// A loaded model and the device its weights live on (features must be built
/// on the same device instance).
struct LoadedModel {
    runner: ProteinMPNNRunner,
    device: Device,
}

#[derive(Default)]
struct MpnnCache {
    protein: Mutex<Option<LoadedModel>>,
    ligand: Mutex<Option<LoadedModel>>,
}

impl MpnnCache {
    /// Lock the slot for `model`, loading the weights on first use.
    ///
    /// The per-model lock is held for load *and* inference: concurrent
    /// requests for the same model wait instead of downloading twice, and
    /// GPU work on one model is serialised. Different models do not block
    /// each other.
    fn acquire(&self, model: MpnnModel) -> Result<MutexGuard<'_, Option<LoadedModel>>> {
        let slot = match model {
            MpnnModel::Protein => &self.protein,
            MpnnModel::Ligand => &self.ligand,
        };
        // A panic mid-load leaves the slot `None`, so a poisoned lock is safe to reuse.
        let mut guard = slot.lock().unwrap_or_else(|e| e.into_inner());
        if guard.is_none() {
            let started = Instant::now();
            let dev = model.device()?;
            let runner = ProteinMPNNRunner::from_pretrained(model.weights(), dev.clone())
                .with_context(|| format!("failed to load {} weights", model.id()))?;
            eprintln!(
                "[mpnn] loaded {} on {:?} in {:.2?}",
                model.id(),
                dev.location(),
                started.elapsed()
            );
            *guard = Some(LoadedModel {
                runner,
                device: dev,
            });
        }
        Ok(guard)
    }
}

/// Tauri-managed cache of loaded MPNN models. Register with `.manage(MpnnState::default())`.
#[derive(Default, Clone)]
pub struct MpnnState {
    cache: Arc<MpnnCache>,
}

// The cache is shared with blocking worker threads.
const _: () = {
    const fn assert_send_sync<T: Send + Sync + 'static>() {}
    assert_send_sync::<MpnnState>();
};

/// Score every residue of a structure with ProteinMPNN or LigandMPNN.
///
/// `model`: `None` picks LigandMPNN when the structure has ligand atoms and
/// ProteinMPNN otherwise; the choice is reported in `DesignLogits.model`.
fn compute_design_logits(
    cache: &MpnnCache,
    structure_text: &str,
    format: StructureFormat,
    model: Option<&str>,
) -> Result<DesignLogits> {
    let prepared = prepare_structure(structure_text, format)?;
    if prepared.residues.is_empty() {
        bail!("no amino-acid residues with a complete backbone (N, CA, C, O) found in structure");
    }
    let model = match model {
        Some(name) => MpnnModel::from_request(name)?,
        None if prepared.n_ligand_atoms > 0 => MpnnModel::Ligand,
        None => MpnnModel::Protein,
    };

    let guard = cache.acquire(model)?;
    let loaded = guard.as_ref().expect("acquire loads the model");

    // Features are built from a file path (the path-based featurizer is the
    // one ferritin's forward pass is verified against), so stage the
    // normalised mmCIF.
    let mut staged = tempfile::Builder::new()
        .prefix("fennec-mpnn-")
        .suffix(".cif")
        .tempfile()
        .context("failed to create staging file")?;
    staged
        .write_all(prepared.staged_cif.as_bytes())
        .context("failed to write staging file")?;
    let exec = MPNNExecConfig::new(
        loaded.device.clone(),
        staged.path().to_string_lossy().into_owned(),
        empty_run_config(),
        None,
        None,
        None,
        None,
        None,
    )?;
    let features = exec
        .generate_protein_features()
        .context("failed to featurize structure")?;

    let r_idx = features
        .get_residue_index()
        .flatten_all()?
        .to_vec1::<u32>()?;
    let seq = features.get_sequence().flatten_all()?.to_vec1::<u32>()?;
    verify_feature_alignment(&prepared.residues, &r_idx, &seq)
        .context("residue mapping check failed")?;

    let log_probs = loaded
        .runner
        .get_log_probs(&features)?
        .to_device(&Device::Cpu)?
        .to_vec2::<f32>()?;
    let device_name = device_name(&loaded.device).to_string();
    drop(guard);

    if log_probs.len() != prepared.residues.len() {
        bail!(
            "model returned {} rows for {} residues",
            log_probs.len(),
            prepared.residues.len()
        );
    }
    if let Some(row) = log_probs.iter().find(|row| row.len() != ALPHABET.len()) {
        bail!(
            "model returned {} columns, expected {}",
            row.len(),
            ALPHABET.len()
        );
    }
    if let Some(i) = log_probs
        .iter()
        .position(|row| row.iter().any(|v| !v.is_finite()))
    {
        bail!(
            "{} returned non-finite values (first at position {i}); refusing to return them",
            model.id()
        );
    }

    let residues = prepared
        .residues
        .into_iter()
        .zip(log_probs)
        .enumerate()
        .map(|(index, (r, logits))| residue_logits(index, r, logits))
        .collect();

    Ok(DesignLogits {
        model: model.id().to_string(),
        alphabet: ALPHABET.iter().map(|c| c.to_string()).collect(),
        residues,
        logits_kind: "log_probs".to_string(),
        uses_ligand_context: model == MpnnModel::Ligand && prepared.n_ligand_atoms > 0,
        n_ligand_atoms: prepared.n_ligand_atoms,
        skipped_residues: prepared.skipped,
        device: device_name,
    })
}

fn device_name(dev: &Device) -> &'static str {
    if dev.is_metal() {
        "metal"
    } else if dev.is_cuda() {
        "cuda"
    } else {
        "cpu"
    }
}

fn residue_logits(index: usize, r: PolymerResidue, logits: Vec<f32>) -> ResidueLogits {
    ResidueLogits {
        index,
        chain_id: r.chain_id,
        auth_seq_id: r.auth_seq_id,
        ins_code: r.ins_code,
        wt: r.wt.to_string(),
        logits,
        label_asym_id: r.label_asym_id,
        label_seq_id: r.label_seq_id,
        comp_id: r.comp_id,
        ligand_distance: r.ligand_distance,
    }
}

fn empty_run_config() -> RunConfig {
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
    }
}

/// Run `f` on a blocking thread and flatten errors (including panics) to strings.
async fn run_blocking<T: Send + 'static>(
    f: impl FnOnce() -> Result<T> + Send + 'static,
) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(f)
        .await
        .map_err(|e| format!("MPNN worker failed: {e}"))?
        .map_err(|e| format!("{e:#}"))
}

/// Per-residue design log-probabilities for a whole structure.
///
/// - `format`: "cif" (default; also "mmcif") or "pdb".
/// - `model`: `None` → LigandMPNN if the structure has ligand atoms, else
///   ProteinMPNN. Accepts the ids from [`mpnn_available_models`] and the
///   aliases "protein_mpnn" / "ligand_mpnn".
///
/// The first call per model loads (and, if not cached, downloads) its weights.
#[tauri::command]
pub async fn mpnn_design_logits(
    state: tauri::State<'_, MpnnState>,
    structure_text: String,
    format: Option<String>,
    model: Option<String>,
) -> Result<DesignLogits, String> {
    let format = super::plm::structure_format(format.as_deref()).map_err(|e| e.to_string())?;
    let cache = state.cache.clone();
    run_blocking(move || compute_design_logits(&cache, &structure_text, format, model.as_deref()))
        .await
}

/// Model ids accepted by [`mpnn_design_logits`].
#[tauri::command]
pub fn mpnn_available_models() -> Vec<String> {
    MpnnModel::ALL.iter().map(|m| m.id().to_string()).collect()
}

/// Legacy single-position endpoint, kept until the frontend migrates.
///
/// Runs ProteinMPNN and returns `softmax(logits / temp)` entries above 1% for
/// model position `position` (0-based; negative → all positions), sorted by
/// probability within each position. The format is sniffed from the text.
#[tauri::command]
pub async fn get_ligmpnn_logits(
    state: tauri::State<'_, MpnnState>,
    pdb_text: String,
    position: i64,
    temp: f32,
) -> Result<Vec<PseudoProbability>, String> {
    let cache = state.cache.clone();
    run_blocking(move || legacy_pseudo_probabilities(&cache, &pdb_text, position, temp)).await
}

fn legacy_pseudo_probabilities(
    cache: &MpnnCache,
    text: &str,
    position: i64,
    temp: f32,
) -> Result<Vec<PseudoProbability>> {
    let design = compute_design_logits(
        cache,
        text,
        StructureFormat::sniff(text),
        Some(MODEL_PROTEIN_MPNN),
    )?;
    let mut out = Vec::new();
    for r in &design.residues {
        if position >= 0 && r.index != position as usize {
            continue;
        }
        let probs = softmax_with_temperature(&r.logits, temp);
        let mut row: Vec<PseudoProbability> = probs
            .into_iter()
            .zip(ALPHABET)
            .filter(|(p, _)| *p > 0.01)
            .map(|(pseudo_prob, amino_acid)| PseudoProbability {
                position: r.index,
                pseudo_prob,
                amino_acid,
            })
            .collect();
        row.sort_by(|a, b| b.pseudo_prob.total_cmp(&a.pseudo_prob));
        out.extend(row);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const CIF: &str = include_str!("../../testdata/structures/101m.cif");
    const PDB: &str = include_str!("../../testdata/structures/1BC8.pdb");

    /// The macOS build must have Metal compiled in and select it by default
    /// (the macOS target dependency enables ferritin-plms/metal). No weights needed.
    #[test]
    #[cfg(target_os = "macos")]
    fn test_default_device_is_metal_on_macos() {
        let dev = device(false).expect("device selection");
        println!(
            "default device: {:?} -> {}",
            dev.location(),
            device_name(&dev)
        );
        assert!(dev.is_metal(), "expected Metal, got {:?}", dev.location());
        assert_eq!(device_name(&MpnnModel::Protein.device().unwrap()), "metal");
        assert_eq!(device_name(&MpnnModel::Ligand.device().unwrap()), "cpu");
    }

    #[test]
    fn test_model_names() {
        assert_eq!(
            mpnn_available_models(),
            vec![
                MODEL_PROTEIN_MPNN.to_string(),
                MODEL_LIGAND_MPNN.to_string()
            ]
        );
        for m in mpnn_available_models() {
            assert_eq!(MpnnModel::from_request(&m).unwrap().id(), m);
        }
        assert_eq!(
            MpnnModel::from_request("protein_mpnn").unwrap(),
            MpnnModel::Protein
        );
        assert_eq!(
            MpnnModel::from_request("ligand_mpnn").unwrap(),
            MpnnModel::Ligand
        );
        assert_eq!(
            MpnnModel::from_request("ligandmpnn-v32-020-25").unwrap(),
            MpnnModel::Ligand
        );
        assert!(MpnnModel::from_request("esm2").is_err());
    }

    fn recovery(d: &DesignLogits) -> f64 {
        let hits = d
            .residues
            .iter()
            .filter(|r| {
                let argmax = r
                    .logits
                    .iter()
                    .enumerate()
                    .max_by(|a, b| a.1.total_cmp(b.1))
                    .unwrap()
                    .0;
                d.alphabet[argmax] == r.wt
            })
            .count();
        hits as f64 / d.residues.len() as f64
    }

    fn check_design(d: &DesignLogits, n_residues: usize) {
        assert_eq!(d.residues.len(), n_residues);
        assert_eq!(d.alphabet.len(), 21);
        for (i, r) in d.residues.iter().enumerate() {
            assert_eq!(r.index, i);
            assert_eq!(r.logits.len(), d.alphabet.len());
            let total: f32 = r.logits.iter().map(|v| v.exp()).sum();
            assert!((total - 1.0).abs() < 1e-3, "row {i} sums to {total}");
        }
    }

    #[test]
    #[ignore = "loads cached HuggingFace weights; run with `cargo test --lib -- --ignored`"]
    fn test_mpnn_design_logits_101m() {
        let cache = MpnnCache::default();
        for (model, label) in [
            (MODEL_PROTEIN_MPNN, "ProteinMPNN"),
            (MODEL_LIGAND_MPNN, "LigandMPNN"),
        ] {
            let t0 = Instant::now();
            let cold =
                compute_design_logits(&cache, CIF, StructureFormat::Cif, Some(model)).unwrap();
            let cold_t = t0.elapsed();
            let t1 = Instant::now();
            let warm =
                compute_design_logits(&cache, CIF, StructureFormat::Cif, Some(model)).unwrap();
            let warm_t = t1.elapsed();
            check_design(&warm, 154);
            assert_eq!(warm.model, model);
            #[cfg(target_os = "macos")]
            assert_eq!(
                warm.device,
                if model == MODEL_PROTEIN_MPNN {
                    "metal"
                } else {
                    "cpu"
                }
            );
            assert_eq!(warm.residues[0].auth_seq_id, 0);
            assert_eq!(warm.residues[0].wt, "M");
            assert_eq!(
                cold.residues[10].logits, warm.residues[10].logits,
                "deterministic"
            );
            let rec = recovery(&warm);
            println!(
                "{label} 101m: {} residues, WT argmax recovery {:.1}%, cold {:.2?}, warm {:.2?}, ligand ctx {}, device {}",
                warm.residues.len(),
                rec * 100.0,
                cold_t,
                warm_t,
                warm.uses_ligand_context,
                warm.device
            );
            assert!(
                rec > 0.25,
                "{label} recovery {rec} is not well above chance (~0.05)"
            );
        }

        // Default picks LigandMPNN because 101m has a heme.
        let d = compute_design_logits(&cache, CIF, StructureFormat::Cif, None).unwrap();
        assert_eq!(d.model, MODEL_LIGAND_MPNN);
        assert!(d.uses_ligand_context);

        // PDB input path (1BC8: zinc finger + DNA).
        let p =
            compute_design_logits(&cache, PDB, StructureFormat::Pdb, Some("ligand_mpnn")).unwrap();
        check_design(&p, p.residues.len());
        println!(
            "LigandMPNN 1BC8: {} residues, WT argmax recovery {:.1}%",
            p.residues.len(),
            recovery(&p) * 100.0
        );

        // Legacy endpoint.
        let all = legacy_pseudo_probabilities(&cache, CIF, -1, 1.0).unwrap();
        assert!(!all.is_empty());
        let single = legacy_pseudo_probabilities(&cache, CIF, 5, 0.1).unwrap();
        assert!(!single.is_empty() && single.iter().all(|p| p.position == 5));
    }
}
