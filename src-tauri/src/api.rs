// use anyhow::Result;
use candle_nn::ops::softmax;
use ferritin_core::AtomCollection;
use ferritin_onnx_models;
use ferritin_onnx_models::LigandMPNN;
use pdbtbx::{Format, ReadOptions};
use serde::Serialize;
use std::io::BufReader;
use tauri::Error as TauriError;

#[derive(Serialize)]
pub struct LIGANDMPNN_LOGITS {
    logits: Vec<f32>,
}

#[tauri::command]
pub fn get_ligmpnn_logits(pdb_text: &str, position: i64) -> Result<LIGANDMPNN_LOGITS, TauriError> {
    let temp = 0.1;
    let pdb_bytes = pdb_text.as_bytes();
    let logits = process_pdb_bytes(pdb_bytes, temp, position)?;
    Ok(LIGANDMPNN_LOGITS { logits: logits })
}

fn process_pdb_bytes(pdb_bytes: &[u8], temp: f32, position: i64) -> anyhow::Result<Vec<f32>> {
    let reader = BufReader::new(pdb_bytes);
    let (pdb, _error) = ReadOptions::default()
        .set_format(Format::Mmcif)
        .read_raw(reader)
        .expect("Failed to parse PDB/CIF");
    let ac = AtomCollection::from(&pdb);
    let model = LigandMPNN::new().unwrap();
    let logits = model.run_model(ac, position, temp)?;
    let logits = candle_nn::ops::softmax(&logits, 1)?;
    let logits = logits.get(0)?.to_vec1()?;
    Ok(logits)
}
