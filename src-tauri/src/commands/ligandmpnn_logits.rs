//! LigandMPNN Logits
use ferritin_core::load_structure_from_string;
use ferritin_plms::types::PseudoProbability;
use ferritin_plms::{ProteinMPNNModels, ProteinMPNNRunner, StructureFeatures, device};

#[tauri::command]
pub fn get_ligmpnn_logits(
    pdb_text: &str,
    position: i64,
    temp: f32,
) -> Result<Vec<PseudoProbability>, String> {
    let _ = temp; // temp unused by simple_decode; retained for API compatibility
    let dev = device(false).map_err(|e| e.to_string())?;

    let ac = load_structure_from_string(pdb_text, "cif").map_err(|e| e.to_string())?;

    let features = ac.featurize_lmpnn(&dev).map_err(|e| format!("{:?}", e))?;

    let runner = ProteinMPNNRunner::load_model(ProteinMPNNModels::V48_020, dev)
        .map_err(|e| e.to_string())?;

    let all_probs = runner
        .get_pseudo_probabilities(&features)
        .map_err(|e| e.to_string())?;

    let result = if position >= 0 {
        all_probs
            .into_iter()
            .filter(|p| p.position == position as usize)
            .collect()
    } else {
        all_probs
    };
    Ok(result)
}
