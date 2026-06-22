//! ESM2 Contact Map / Logits
use anyhow::Result;
use ferritin_core::load_structure_from_string;
use ferritin_plms::featurize::utilities::aa3to1;
use ferritin_plms::types::PseudoProbability;
use ferritin_plms::{ESM2Models, ESM2Runner, device};

#[tauri::command]
pub fn get_esm2_logits(pdb_seq: &str) -> Result<Vec<PseudoProbability>, String> {
    let prot_seq = pdb_to_sequence(pdb_seq).map_err(|e| e.to_string())?;
    let dev = device(false).map_err(|e| e.to_string())?;
    let runner = ESM2Runner::load_model(ESM2Models::T6_8M, dev)
        .map_err(|e| e.to_string())?;
    runner.get_pseudo_probabilities(&prot_seq).map_err(|e| e.to_string())
}

fn pdb_to_sequence(prot_seq: &str) -> Result<String> {
    let ac = load_structure_from_string(prot_seq, "cif")?;
    let sequence = ac
        .iter_residues_aminoacid()
        .map(|res| res.residue_name().to_string())
        .map(|res3| aa3to1(&res3))
        .collect::<String>();
    Ok(sequence)
}
