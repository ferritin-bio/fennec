//! Amplify CLI
use ferritin_plms::{AmplifyModels, AmplifyRunner}
use tauri;
use pdbtbx::{Format, ReadOptions};


#[tauri::command]
pub fn get_ligmpnn_logits(pdb_text: &str, position: i64, temp: f32) -> Result<LIGANDMPNN_LOGITS, TauriError> {
    let pdb_bytes = pdb_text.as_bytes();
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
    let mut amino_acid_probs = Vec::new();
    for i in 0..21 {
        amino_acid_probs.push(serde_json::json!({
            "amino_acid": int_to_aa1(i).to_string(),
            "pseudo_prob": logits[i as usize]
        }));
    }

    Ok(LIGANDMPNN_LOGITS { amino_acid_probs })
}
