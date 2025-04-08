//! LigandMPNN_Logits
//!
use anyhow::Result;
use ferritin_core::load_structure_from_string;
use ferritin_onnx_models;
use ferritin_onnx_models::LigandMPNN;
use ferritin_plms::types::PseudoProbability;
use tauri;
use tauri::Error as TauriError;

#[tauri::command]
pub fn get_ligmpnn_logits(
    pdb_text: &str,
    position: i64,
    temp: f32,
) -> Result<Vec<PseudoProbability>, TauriError> {
    let ac = match load_structure_from_string(pdb_text, "cif") {
        Ok(ac) => ac,
        Err(e) => return Err(tauri::Error::Anyhow(e)),
    };
    let model = LigandMPNN::new()?;
    let outputs = model.get_single_location(ac, temp, position)?;
    Ok(outputs)
}
