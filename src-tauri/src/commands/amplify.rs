//! AMPLIFY logits and attention contact maps.
use ferritin_plms::types::{ContactMap, PseudoProbability};

use super::plm::{PlmState, chain_sequence, run_blocking, structure_format};

/// Per-position AMPLIFY 120M pseudo-probabilities for one protein chain.
///
/// - `format`: "cif" (default; mmCIF) or "pdb".
/// - `chain_id`: author chain id; `None` scores the first protein chain.
#[tauri::command]
pub async fn get_amplify_logits(
    state: tauri::State<'_, PlmState>,
    structure_text: String,
    format: Option<String>,
    chain_id: Option<String>,
) -> Result<Vec<PseudoProbability>, String> {
    let state = state.inner().clone();
    run_blocking(move || {
        let fmt = structure_format(format.as_deref())?;
        let chain = chain_sequence(&structure_text, fmt, chain_id.as_deref())?;
        state.with_amplify(|runner| runner.get_pseudo_probabilities(&chain.sequence))
    })
    .await
}

/// AMPLIFY 120M attention-derived contact map for one protein chain.
///
/// Same arguments as [`get_amplify_logits`].
#[tauri::command]
pub async fn get_amplify_contact_map(
    state: tauri::State<'_, PlmState>,
    structure_text: String,
    format: Option<String>,
    chain_id: Option<String>,
) -> Result<Vec<ContactMap>, String> {
    let state = state.inner().clone();
    run_blocking(move || {
        let fmt = structure_format(format.as_deref())?;
        let chain = chain_sequence(&structure_text, fmt, chain_id.as_deref())?;
        state.with_amplify(|runner| runner.get_contact_map(&chain.sequence))
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::super::mpnn_structure::StructureFormat;
    use super::*;

    const CIF: &str = include_str!("../../testdata/structures/101m.cif");

    fn sequence() -> String {
        chain_sequence(CIF, StructureFormat::Cif, None)
            .unwrap()
            .sequence
    }

    #[test]
    #[ignore = "loads cached HuggingFace weights; run with `cargo test --lib -- --ignored`"]
    fn test_amplify_logits_from_cif() {
        let state = PlmState::default();
        let logits = state
            .with_amplify(|r| r.get_pseudo_probabilities(&sequence()))
            .expect("AMPLIFY logits");
        assert!(!logits.is_empty());
        assert!(
            logits
                .iter()
                .all(|p| p.pseudo_prob.is_finite() && (0.0..=1.0).contains(&p.pseudo_prob))
        );
    }

    /// Regression test for fennec-9lm.7: the contact map used to parse mmCIF as PDB.
    #[test]
    #[ignore = "loads cached HuggingFace weights; run with `cargo test --lib -- --ignored`"]
    fn test_amplify_contact_map_from_cif() {
        let state = PlmState::default();
        let seq = sequence();
        let t0 = std::time::Instant::now();
        let contacts = state
            .with_amplify(|r| r.get_contact_map(&seq))
            .expect("AMPLIFY contact map");
        let cold = t0.elapsed();
        let t1 = std::time::Instant::now();
        state.with_amplify(|r| r.get_contact_map(&seq)).unwrap();
        println!(
            "AMPLIFY contact map 101m: cold {cold:.2?}, warm (cached) {:.2?}",
            t1.elapsed()
        );
        assert!(!contacts.is_empty());
        assert!(contacts.iter().all(|c| c.contact_estimate.is_finite()));
        let max_pos = contacts
            .iter()
            .map(|c| c.position_1.max(c.position_2))
            .max()
            .unwrap();
        assert!(max_pos < seq.len(), "positions index the chain sequence");
    }
}
