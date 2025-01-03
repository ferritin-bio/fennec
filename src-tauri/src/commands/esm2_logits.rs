//! ESM2 Contact Map
//!
use anyhow::{Error as E, Result};
use ferritin_core::AtomCollection;
use ferritin_onnx_models::{ESM2Models, LogitPosition, ESM2};
use pdbtbx::{Format, ReadOptions};
use std::io::BufReader;

#[rustfmt::skip]
// todo: better utility library
pub fn aa3to1(aa: &str) -> char {
    match aa {
        "ALA" => 'A', "CYS" => 'C', "ASP" => 'D',
        "GLU" => 'E', "PHE" => 'F', "GLY" => 'G',
        "HIS" => 'H', "ILE" => 'I', "LYS" => 'K',
        "LEU" => 'L', "MET" => 'M', "ASN" => 'N',
        "PRO" => 'P', "GLN" => 'Q', "ARG" => 'R',
        "SER" => 'S', "THR" => 'T', "VAL" => 'V',
        "TRP" => 'W', "TYR" => 'Y', _     => 'X',
    }
}

#[tauri::command]
pub fn get_esm2_logits(pdb_seq: &str) -> Result<Vec<LogitPosition>, String> {
    let prot_seq = pdb_to_sequence(pdb_seq).map_err(|e| e.to_string())?;
    let esm_model = ESM2Models::ESM2_T6_8M;
    // let esm_model = ESM2Models::ESM2_T12_35M;
    // let esm_model = ESM2Models::ESM2_T30_150M;
    // let esm_model = ESM2Models::ESM2_T33_650M;

    let esm2 = ESM2::new(esm_model).map_err(|e| e.to_string())?;
    let logits = esm2.run_model(&prot_seq).map_err(|e| e.to_string())?;
    let normed = esm2.extract_logits(&logits).map_err(|e| e.to_string())?;
    // println!("Normed: {:?}", normed);
    Ok(normed)
}

fn pdb_to_sequence(prot_seq: &str) -> Result<String> {
    let reader = BufReader::new(prot_seq.as_bytes());
    let (pdb, _error) = ReadOptions::default()
        .set_format(Format::Mmcif)
        .read_raw(reader)
        .expect("Failed to parse PDB/CIF");
    let ac = AtomCollection::from(&pdb);
    let sequence = ac
        .iter_residues_aminoacid()
        .map(|res| res.res_name)
        .map(|res3| aa3to1(&res3))
        .collect::<String>();

    println!("Sequence: {:?}", sequence);
    Ok(sequence)
}
