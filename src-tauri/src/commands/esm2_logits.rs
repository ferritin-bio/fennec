//! ESM2 Contact Map
//!
use anyhow::{Error as E, Result};
use ferritin_onnx_models::{ESM2Models, LogitPosition, ESM2};
use ndarray::Array2;
use ort::{
    execution_providers::CUDAExecutionProvider,
    session::{builder::GraphOptimizationLevel, Session},
};
use std::env;

#[tauri::command]
pub fn get_esm2_logits(prot_seq: &str) -> Result<Vec<LogitPosition>, String> {
    let esm_model = ESM2Models::ESM2_T6_8M;
    // let esm_model = ESM2Models::ESM2_T12_35M;
    // let esm_model = ESM2Models::ESM2_T30_150M;
    // let esm_model = ESM2Models::ESM2_T33_650M;

    let esm2 = ESM2::new(esm_model).map_err(|e| e.to_string())?;
    let logits = esm2.run_model(prot_seq).map_err(|e| e.to_string())?;
    let normed = esm2.extract_logits(&logits).map_err(|e| e.to_string())?;
    Ok(normed)
}
