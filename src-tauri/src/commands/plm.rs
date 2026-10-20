//! Shared plumbing for the sequence language models (ESM2, AMPLIFY).
//!
//! Structures arrive as mmCIF by default — the current PDB archive standard and what
//! the frontend fetches from RCSB / AlphaFold DB. Legacy PDB text is still accepted
//! when the caller says `format: "pdb"`. Both go through the same reader as the MPNN
//! commands (`mpnn_structure::prepare_structure`), so every model sees the same
//! residues: first model only, one alt-loc, insertion codes kept, waters dropped.
//!
//! Loaded models are cached in [`PlmState`] and inference runs off the UI thread.
use anyhow::{Context, Result, bail};
use ferritin_plms::{AmplifyModels, AmplifyRunner, ESM2Models, ESM2Runner, device};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Instant;

use super::mpnn_structure::{StructureFormat, prepare_structure};

/// Parse a `format` argument, defaulting to mmCIF.
pub(crate) fn structure_format(format: Option<&str>) -> Result<StructureFormat> {
    StructureFormat::parse(format.unwrap_or("cif"))
}

/// One protein chain's sequence, as the models see it.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ChainSequence {
    /// Author chain id (`auth_asym_id`).
    pub chain_id: String,
    /// One-letter sequence; non-canonical residues are 'X'.
    pub sequence: String,
}

/// The sequence of `chain_id` (author chain id), or of the first protein chain when `None`.
///
/// Sequence models are run per chain: concatenating chains would invent peptide
/// bonds between them.
pub(crate) fn chain_sequence(
    text: &str,
    format: StructureFormat,
    chain_id: Option<&str>,
) -> Result<ChainSequence> {
    let prepared = prepare_structure(text, format)?;
    let chain = match chain_id {
        Some(c) => c.to_string(),
        None => prepared.residues[0].chain_id.clone(),
    };
    let sequence: String = prepared
        .residues
        .iter()
        .filter(|r| r.chain_id == chain)
        .map(|r| r.wt)
        .collect();
    if sequence.is_empty() {
        let mut chains: Vec<&str> = prepared
            .residues
            .iter()
            .map(|r| r.chain_id.as_str())
            .collect();
        chains.dedup();
        bail!("no protein residues in chain {chain:?}; protein chains: {chains:?}");
    }
    Ok(ChainSequence {
        chain_id: chain,
        sequence,
    })
}

/// Tauri-managed cache of loaded sequence models. Register with `.manage(PlmState::default())`.
#[derive(Default, Clone)]
pub struct PlmState {
    cache: Arc<PlmCache>,
}

#[derive(Default)]
struct PlmCache {
    esm2: Mutex<Option<ESM2Runner>>,
    amplify: Mutex<Option<AmplifyRunner>>,
}

// The cache is shared with blocking worker threads.
const _: () = {
    const fn assert_send_sync<T: Send + Sync + 'static>() {}
    assert_send_sync::<PlmState>();
};

/// Lock `slot`, loading the model on first use. The lock is held for load and
/// inference, so a model never downloads twice or runs concurrently on the GPU.
/// A panic mid-load leaves the slot `None`, so a poisoned lock is safe to reuse.
fn acquire<'a, T>(
    slot: &'a Mutex<Option<T>>,
    name: &str,
    load: impl FnOnce() -> Result<T>,
) -> Result<MutexGuard<'a, Option<T>>> {
    let mut guard = slot.lock().unwrap_or_else(|e| e.into_inner());
    if guard.is_none() {
        let started = Instant::now();
        *guard = Some(load().with_context(|| format!("failed to load {name} weights"))?);
        eprintln!("[plm] loaded {name} in {:.2?}", started.elapsed());
    }
    Ok(guard)
}

impl PlmState {
    pub(crate) fn with_esm2<R>(&self, f: impl FnOnce(&ESM2Runner) -> Result<R>) -> Result<R> {
        let guard = acquire(&self.cache.esm2, "ESM2 t6_8M", || {
            ESM2Runner::from_pretrained(ESM2Models::T6_8M, device(false)?)
        })?;
        f(guard.as_ref().expect("loaded above"))
    }

    pub(crate) fn with_amplify<R>(&self, f: impl FnOnce(&AmplifyRunner) -> Result<R>) -> Result<R> {
        let guard = acquire(&self.cache.amplify, "AMPLIFY 120M", || {
            AmplifyRunner::from_pretrained(AmplifyModels::AMP120M, device(false)?)
        })?;
        f(guard.as_ref().expect("loaded above"))
    }
}

/// Run `f` on a blocking thread and flatten errors (including panics) to strings.
pub(crate) async fn run_blocking<T: Send + 'static>(
    f: impl FnOnce() -> Result<T> + Send + 'static,
) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(f)
        .await
        .map_err(|e| format!("model worker failed: {e}"))?
        .map_err(|e| format!("{e:#}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const CIF_101M: &str = include_str!("../../testdata/structures/101m.cif");
    const PDB_1BC8: &str = include_str!("../../testdata/structures/1BC8.pdb");

    #[test]
    fn test_format_defaults_to_mmcif() {
        assert_eq!(structure_format(None).unwrap(), StructureFormat::Cif);
        assert_eq!(structure_format(Some("pdb")).unwrap(), StructureFormat::Pdb);
        assert!(structure_format(Some("sdf")).is_err());
    }

    #[test]
    fn test_chain_sequence_from_mmcif() {
        let s = chain_sequence(CIF_101M, StructureFormat::Cif, None).unwrap();
        assert_eq!(s.chain_id, "A");
        assert_eq!(s.sequence.len(), 154);
        assert!(s.sequence.starts_with("MVLSEGEWQLVLHVWAKVEAD"));
    }

    #[test]
    fn test_chain_sequence_selects_chain_and_reports_missing() {
        let first = chain_sequence(PDB_1BC8, StructureFormat::Pdb, None).unwrap();
        let explicit =
            chain_sequence(PDB_1BC8, StructureFormat::Pdb, Some(&first.chain_id)).unwrap();
        assert_eq!(first, explicit);
        let err = chain_sequence(PDB_1BC8, StructureFormat::Pdb, Some("ZZ")).unwrap_err();
        assert!(err.to_string().contains("protein chains"), "{err}");
    }
}
