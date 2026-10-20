//! ESM2 logits.
use ferritin_plms::types::PseudoProbability;

use super::plm::{PlmState, chain_sequence, run_blocking, structure_format};

/// Per-position ESM2 (t6_8M) pseudo-probabilities for one protein chain.
///
/// - `format`: "cif" (default; mmCIF) or "pdb".
/// - `chain_id`: author chain id; `None` scores the first protein chain.
#[tauri::command]
pub async fn get_esm2_logits(
    state: tauri::State<'_, PlmState>,
    structure_text: String,
    format: Option<String>,
    chain_id: Option<String>,
) -> Result<Vec<PseudoProbability>, String> {
    let state = state.inner().clone();
    run_blocking(move || {
        let fmt = structure_format(format.as_deref())?;
        let chain = chain_sequence(&structure_text, fmt, chain_id.as_deref())?;
        state.with_esm2(|runner| runner.get_pseudo_probabilities(&chain.sequence))
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::super::mpnn_structure::StructureFormat;
    use super::*;

    const CIF: &str = include_str!("../../testdata/structures/101m.cif");

    #[test]
    #[ignore = "loads cached HuggingFace weights; run with `cargo test --lib -- --ignored`"]
    fn test_esm2_logits_from_cif() {
        let seq = chain_sequence(CIF, StructureFormat::Cif, None)
            .unwrap()
            .sequence;
        let state = PlmState::default();
        let t0 = std::time::Instant::now();
        let logits = state
            .with_esm2(|r| r.get_pseudo_probabilities(&seq))
            .expect("ESM2 logits");
        let cold = t0.elapsed();
        let t1 = std::time::Instant::now();
        state
            .with_esm2(|r| r.get_pseudo_probabilities(&seq))
            .unwrap();
        println!(
            "ESM2 101m: cold {cold:.2?}, warm (cached) {:.2?}",
            t1.elapsed()
        );
        assert!(!logits.is_empty());
        assert!(
            logits
                .iter()
                .all(|p| p.pseudo_prob.is_finite() && (0.0..=1.0).contains(&p.pseudo_prob))
        );
    }
}
