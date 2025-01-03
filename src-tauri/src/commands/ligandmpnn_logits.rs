//! LigandMPNN_Logits
//!
use ferritin_core::AtomCollection;
use ferritin_onnx_models;
use ferritin_onnx_models::LigandMPNN;
use ferritin_plms::types::PseudoProbability;
use pdbtbx::{Format, ReadOptions};
use std::io::BufReader;
use tauri;
use tauri::Error as TauriError;

#[tauri::command]
pub fn get_ligmpnn_logits(
    pdb_text: &str,
    position: i64,
    temp: f32,
) -> Result<Vec<PseudoProbability>, TauriError> {
    let pdb_bytes = pdb_text.as_bytes();
    let reader = BufReader::new(pdb_bytes);
    let (pdb, _error) = ReadOptions::default()
        .set_format(Format::Mmcif)
        .read_raw(reader)
        .expect("Failed to parse PDB/CIF");
    let ac = AtomCollection::from(&pdb);
    let model = LigandMPNN::new().unwrap();
    let outputs = model.get_single_location(ac, temp, position)?;
    Ok(outputs)
}
