//! LigandMPNN Logits
use ferritin_plms::ligandmpnn::configs::{MPNNExecConfig, ModelTypes, RunConfig};
use ferritin_plms::types::PseudoProbability;
use ferritin_plms::{ProteinMPNNModels, ProteinMPNNRunner, device};
use std::io::Write;

#[tauri::command]
pub fn get_ligmpnn_logits(
    pdb_text: &str,
    position: i64,
    temp: f32,
) -> Result<Vec<PseudoProbability>, String> {
    let _ = temp; // temp unused by simple_decode; retained for API compatibility
    let dev = device(false).map_err(|e| e.to_string())?;

    // Features are built from a file path (the path-based featurizer is the one
    // ferritin's ProteinMPNN forward pass is verified against), so stage the CIF text.
    let mut structure_file = tempfile::Builder::new()
        .suffix(".cif")
        .tempfile()
        .map_err(|e| e.to_string())?;
    structure_file
        .write_all(pdb_text.as_bytes())
        .map_err(|e| e.to_string())?;
    let structure_path = structure_file.path().to_string_lossy().into_owned();

    let run_config = RunConfig {
        model_type: Some(ModelTypes::ProteinMPNN),
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
    };
    let exec = MPNNExecConfig::new(
        dev.clone(),
        structure_path,
        run_config,
        None,
        None,
        None,
        None,
        None,
    )
    .map_err(|e| format!("{:?}", e))?;
    let features = exec
        .generate_protein_features()
        .map_err(|e| format!("{:?}", e))?;

    let runner = ProteinMPNNRunner::from_pretrained(ProteinMPNNModels::V48_020, dev)
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

#[cfg(test)]
mod tests {
    use super::*;

    const CIF: &str = include_str!("../../testdata/structures/101m.cif");

    #[test]
    #[ignore = "loads cached HuggingFace weights; run with `cargo test --lib -- --ignored`"]
    fn test_ligandmpnn_logits_all_and_single_position() {
        let all = get_ligmpnn_logits(CIF, -1, 0.1).expect("LigandMPNN logits");
        assert!(!all.is_empty());

        let position = all[0].position as i64;
        let single = get_ligmpnn_logits(CIF, position, 0.1).expect("LigandMPNN single position");
        assert!(!single.is_empty());
        assert!(single.iter().all(|p| p.position as i64 == position));
    }
}
