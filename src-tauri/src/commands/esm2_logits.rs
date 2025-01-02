//! ESM2 Contact Map
//!
use anyhow::Error as E;
use ferritin_onnx_models::{ESM2Models, ESM2};
use ndarray::Array2;
use ort::{
    execution_providers::CUDAExecutionProvider,
    session::{builder::GraphOptimizationLevel, Session},
};
use std::env;

#[derive(Serialize)]
pub struct ESM2_LOGITS {
    amino_acid_probs: Vec<serde_json::Value>,
}

#[tauri::command]
pub fn get_esm2_logits(prot_seq: &str) {
    let esm_model = ESM2Models::ESM2_T6_8M;
    // let esm_model = ESM2Models::ESM2_T12_35M;
    // let esm_model = ESM2Models::ESM2_T30_150M;
    // let esm_model = ESM2Models::ESM2_T33_650M;
    let esm_model = ESM2Models::ESM2_T6_8M;
    let esm2 = ESM2::new(esm_model)?;
    let protein = args.protein_string.as_ref().unwrap().as_str();
    let logits = esm2.run_model(protein)?;
    let normed = esm2.extract_logits(&logits)?;
    let logits = candle_nn::ops::softmax(&logits, 1)?;
    let logits = logits.get(0)?.to_vec1()?;
    Ok(logits)
}
