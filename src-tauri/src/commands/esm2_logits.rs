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
    let runner = ESM2Runner::from_pretrained(ESM2Models::T6_8M, dev).map_err(|e| e.to_string())?;
    let result = runner
        .get_pseudo_probabilities(&prot_seq)
        .map_err(|e| e.to_string())?;
    Ok(result)
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

#[cfg(test)]
mod tests {
    use super::*;

    // get_esm2_logits parses mmCIF text, matching what the frontend sends
    const CIF: &str = include_str!("../../testdata/structures/101m.cif");

    #[test]
    #[ignore = "loads cached HuggingFace weights; run with `cargo test --lib -- --ignored`"]
    fn test_esm2_logits_from_cif() {
        let logits = get_esm2_logits(CIF).expect("ESM2 logits");
        assert!(!logits.is_empty());
        assert!(
            logits
                .iter()
                .all(|p| p.pseudo_prob.is_finite() && (0.0..=1.0).contains(&p.pseudo_prob))
        );
    }
}
