//! ESM2 Contact Map
//!
use anyhow::Result;
use ferritin_core::load_structure_from_string;
use ferritin_onnx_models::{ESM2, ESM2Models};
use ferritin_plms::ligandmpnn::utilities::aa3to1;
use ferritin_plms::types::PseudoProbability;

#[tauri::command]
pub fn get_esm2_logits(pdb_seq: &str) -> Result<Vec<PseudoProbability>, String> {
    let prot_seq = pdb_to_sequence(pdb_seq).map_err(|e| e.to_string())?;
    let esm_model = ESM2Models::ESM2_T6_8M;
    // let esm_model = ESM2Models::ESM2_T12_35M;
    // let esm_model = ESM2Models::ESM2_T30_150M;
    // let esm_model = ESM2Models::ESM2_T33_650M;

    let esm2 = ESM2::new(esm_model).map_err(|e| e.to_string())?;
    let logits = esm2.run_model(&prot_seq).map_err(|e| e.to_string())?;
    let normed = esm2.extract_logits(&logits).map_err(|e| e.to_string())?;
    let filtered = normed
        .into_iter()
        .filter(|logit| logit.amino_acid.is_alphabetic())
        .filter(|logit| logit.amino_acid != 'X')
        .filter(|logit| logit.amino_acid != 'B')
        .filter(|logit| logit.amino_acid != 'Z')
        .collect();
    Ok(filtered)
}

fn pdb_to_sequence(prot_seq: &str) -> Result<String> {
    let ac = load_structure_from_string(prot_seq, "cif")?;
    let sequence = ac
        .iter_residues_aminoacid()
        .map(|res| res.residue_name().to_string())
        .map(|res3| aa3to1(&res3))
        .collect::<String>();

    println!("Sequence: {:?}", sequence);
    Ok(sequence)
}
