# UI overhaul: Mol* v6 structure-design workspace

Status of each item: ✅ in this branch · 🟡 partial · ⬜ proposed.

## Why

The current structure pages fetch from RCSB three separate times, use Miew with a
global `window.miew_viewer` (and leak a WebGL context on every navigation), re-run
LigandMPNN on every click *and* every temperature-slider tick (temperature is ignored
by the backend), and block the main thread during inference. The overhaul centres the
app on one structure workspace: load once, run a model once, then explore instantly.

## Architecture

```
StructureSource (PDB / AlphaFold / file) ──► structure store ──► Mol* v6 viewer (Svelte-owned chrome)
                                                   │                  │ hover / click (auth chain + seq id)
                                                   ▼                  ▼
                                   mpnn_design_logits (async, cached runner)   selection store
                                                   │                  │
                                                   ▼                  ▼
                              DesignLogits (L × A raw logits) ──► softmax(T) on the frontend
                                                   │
                     ┌─────────────────────────────┼──────────────────────────┐
                     ▼                             ▼                          ▼
           hover substitution card      colour-by-metric theme        sequence heat-strip
```

- **Mol* v6** comes from the GitHub head (`6.0.0-dev.0`), a pnpm monorepo. We build it
  and install the packed `@molstar/*` tarballs from `vendor/molstar/`
  (`scripts/vendor-molstar.sh` pins the commit). The headless `PluginContext` is
  mounted without the React UI, so Fennec's Svelte components own every control.
- **One model call per structure.** `mpnn_design_logits` returns raw logits for every
  residue, keyed by author chain / author seq id / insertion code, so they line up with
  Mol* picks. Temperature, entropy, P(WT) and mutation pressure are computed in
  `src/lib/design/logits.ts`, so changing the slider is instant.

## Structure input: mmCIF first

mmCIF is the PDB archive standard, so every structure command (`mpnn_design_logits`,
`get_esm2_logits`, `get_amplify_logits`, `get_amplify_contact_map`) takes
`structure_text` plus an optional `format` that defaults to `"cif"`; legacy `"pdb"` is
still accepted. All of them read structures through `mpnn_structure::prepare_structure`
(first model, one alt-loc, insertion codes kept, waters dropped), so every model sees the
same residues. Sequence models run on one chain (`chain_id`, default the first protein
chain) rather than concatenating chains.

## Feature list (ranked by value ÷ effort)

| # | Feature | Status |
|---|---------|--------|
| 1 | Shared structure store + source picker (PDB ID, AlphaFold DB, local file) | ✅ PDB / AFDB / file |
| 2 | Backend contract: async commands, cached model runners, full-matrix LigandMPNN with residue mapping | ✅ MPNN, ESM2, AMPLIFY (all mmCIF-first) |
| 3 | Mol* v6 viewer component (headless plugin, Svelte wrapper, hover/click → selection store, disposal) | ✅ |
| 4 | Workspace layout: viewer ‖ analysis panel, sequence strip below | ✅ |
| 5 | **Hover substitution card**: hover any residue and see its distribution over the 20 amino acids at the current T | ✅ |
| 6 | **Colour structure by metric**: entropy, P(WT), mutation pressure, or P(chosen AA) for every residue, via a custom Mol* colour theme + legend | ✅ |
| 7 | **"Co-express" substitution density**: pick an amino acid (e.g. W) and paint every residue by P(W). Hovering a bar in the card repaints the structure with that AA | ✅ |
| 8 | Sequence heat-strip (position × AA logits) cross-highlighted with the 3D view | ✅ strip; ⬜ brushing a range |
| 9 | Mutation basket: click a bar to add `A123V`, then export CSV / FASTA | ⬜ |
| 10 | Model comparison at one site (LigandMPNN vs ESM2 vs AMPLIFY small multiples + agreement score) | ⬜ |
| 11 | Contact map ↔ structure linking (click a contact → distance line in Mol*; predicted vs Cβ distance) | ⬜ |
| 12 | Job/progress UI (toast, progress, cancel) | 🟡 inline status only |
| 13 | Dark mode wired to tokens (`.dark` is never applied today) + chart tokens + Mol* background from theme | 🟡 Mol* background follows theme |
| 14 | Session persistence (recent structures, on-disk result cache) | ⬜ |
| 15 | Command palette (go to `A:123`, load PDB, switch colour mode) | ⬜ |
| 16 | Prune / repair the bioinformatics section (duplicate GenomeRibbon, dead `/settings` + `/help`, empty Ketcher) | ⬜ |

## Bugs found by the frontend scan (not fixed in this branch unless marked)

- ~~`get_amplify_contact_map` parses `"pdb"` but the frontend sends mmCIF.~~ Fixed: every structure command now defaults to mmCIF and shares one reader.
- `app-sidebar.svelte` has a `:global(svg){width:1.2em}` rule that shrinks every SVG in the app.
- `<main class="container">` in `+layout.svelte` centres the whole app, which is why the sidebar needs `text-align:left !important`.
- `FastaStats` appends plots on each resize; `RnaVisualizer` writes into a node before it exists on the first render.
- `GenomeRibbon` is a copy of the MSA viewer; `/settings` and `/help` 404.
