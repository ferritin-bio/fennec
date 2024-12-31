// use anyhow::Result;
use candle_nn::ops::softmax;
use ferritin_core::AtomCollection;
use ferritin_onnx_models;
use ferritin_onnx_models::LigandMPNN;
use pdbtbx::{Format, ReadOptions};
use serde::Serialize;
use std::io::BufReader;
use tauri::Error as TauriError;



// lifted from ferritin-plms::ligandMPNN
//
pub fn int_to_aa1(aa_int: u32) -> char {
    match aa_int {
        0 => 'A', 1 => 'C', 2 => 'D',
        3 => 'E', 4 => 'F', 5 => 'G',
        6 => 'H', 7 => 'I', 8 => 'K',
        9 => 'L', 10 => 'M', 11 => 'N',
        12 => 'P', 13 => 'Q', 14 => 'R',
        15 => 'S', 16 => 'T', 17 => 'V',
        18 => 'W', 19 => 'Y', 20 => 'X',
        _ => 'X'
    }
}

#[derive(Serialize)]
pub struct LIGANDMPNN_LOGITS {
    amino_acid_probs: Vec<serde_json::Value>
}


#[tauri::command]
pub fn get_ligmpnn_logits(pdb_text: &str, position: i64, temp: f32) -> Result<LIGANDMPNN_LOGITS, TauriError> {
    let pdb_bytes = pdb_text.as_bytes();
    let logits = process_pdb_bytes(pdb_bytes, temp, position)?;
    let mut amino_acid_probs = Vec::new();
    for i in 0..21 {
        amino_acid_probs.push(serde_json::json!({
            "amino_acid": int_to_aa1(i).to_string(),
            "pseudo_prob": logits[i as usize]
        }));
    }

    Ok(LIGANDMPNN_LOGITS { amino_acid_probs })
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
