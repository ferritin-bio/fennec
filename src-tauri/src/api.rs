use anyhow::Result;
use ferritin_core::AtomCollection;
use ferritin_onnx_models;
use ferritin_onnx_models::LigandMPNN;
use pdbtbx::{Format, ReadOptions};
use serde::Serialize;
use std::io::BufReader;
use std::io::Read;

#[derive(Serialize)]
pub struct LIGANDMPNN_LOGITS {
    sequence: Vec<f32>,
}

#[tauri::command]
pub fn get_ligmpnn_logits(name: &str, position: i64) -> Result<LIGANDMPNN_LOGITS> {
    let temp = 0.1;
    let pos = 10;
    let pdb_bytes = name.as_bytes();
    let logits = process_pdb_bytes(pdb_bytes, temp, pos)?;
    Ok(LIGANDMPNN_LOGITS { sequence: logits })
}

fn process_pdb_bytes(pdb_bytes: &[u8], temp: f32, pos: i64) -> Result<Vec<f32>> {
    let reader = BufReader::new(pdb_bytes);
    let (pdb, _error) = ReadOptions::default()
        .set_format(Format::Mmcif)
        .read_raw(reader)
        .expect("Failed to parse PDB/CIF");
    let ac = AtomCollection::from(&pdb);
    let model = LigandMPNN::new().unwrap();
    let logits = model.run_model(ac, pos, temp)?;
    let logits = logits.to_vec1()?;
    Ok(logits)
}
