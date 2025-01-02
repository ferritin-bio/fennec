//! ESM2 Contact Map
//!
use anyhow::{Error as E, Result};
use ferritin_core::AtomCollection;
use ferritin_onnx_models::{ESM2Models, LogitPosition, ESM2};
use ort::{
    execution_providers::CUDAExecutionProvider,
    session::{builder::GraphOptimizationLevel, Session},
};
use pdbtbx::{Format, ReadOptions};
use std::env;
use std::io::BufReader;

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
    Ok(normed)
}

fn pdb_to_sequence(prot_seq: &str) -> Result<String> {
    let reader = BufReader::new(prot_seq.as_bytes());
    let (pdb, _error) = ReadOptions::default()
        .set_format(Format::Mmcif)
        .read_raw(reader)
        .expect("Failed to parse PDB/CIF");
    let ac = AtomCollection::from(&pdb);
    Ok(ac.get_resnames().join(""))
}
