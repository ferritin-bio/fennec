/**
 * Design-model results for the loaded structure. The backend is called once per
 * (structure, model); temperature and colour metrics are derived locally.
 */
import { invoke } from "@tauri-apps/api/core";
import { errorMessage } from "#lib/utils.js";
import type { LoadedStructure } from "#lib/structure/store.svelte.js";
import {
  metricValues,
  residueKey,
  summarize,
  type ColorMetric,
  type DesignLogits,
  type ResidueKey,
  type ResidueSummary,
} from "./logits.js";

/** "auto" lets the backend pick LigandMPNN when the structure has ligand atoms. */
export type DesignModel = "auto" | "protein_mpnn" | "ligand_mpnn";

class DesignStore {
  model = $state<DesignModel>("auto");
  data = $state.raw<DesignLogits | null>(null);
  loading = $state(false);
  error = $state<string | null>(null);
  elapsedMs = $state<number | null>(null);

  temperature = $state(0.5);
  metric = $state<ColorMetric>("entropy");
  /** Amino acid painted by the "substitution" metric. */
  focusAA = $state("W");
  /** Transient AA preview while hovering a bar in the substitution card. */
  previewAA = $state<string | null>(null);

  summaries: ResidueSummary[] = $derived(this.data ? summarize(this.data, this.temperature) : []);
  byKey: Map<string, ResidueSummary> = $derived(new Map(this.summaries.map((s) => [residueKey(s), s])));

  /** The metric actually painted: a hovered bar temporarily overrides the chosen metric. */
  activeMetric: ColorMetric = $derived(this.previewAA ? "substitution" : this.metric);
  activeAA: string = $derived(this.previewAA ?? this.focusAA);

  colorValues: Map<string, number> = $derived(
    this.data ? metricValues(this.summaries, this.data.alphabet, this.activeMetric, this.activeAA) : new Map(),
  );

  private requestId = 0;
  private cache = new Map<string, DesignLogits>();

  summaryFor(r: ResidueKey | null): ResidueSummary | undefined {
    return r ? this.byKey.get(residueKey(r)) : undefined;
  }

  reset() {
    this.data = null;
    this.error = null;
    this.elapsedMs = null;
    this.previewAA = null;
  }

  async run(s: LoadedStructure) {
    const cacheKey = `${this.model}|${s.source}|${s.id}|${s.text.length}`;
    const cached = this.cache.get(cacheKey);
    if (cached) {
      this.data = cached;
      return;
    }
    const id = ++this.requestId;
    if (!("__TAURI_INTERNALS__" in window)) {
      this.error = "Design models run in the Fennec desktop app (npm run tauri dev), not a plain browser.";
      return;
    }
    this.loading = true;
    this.error = null;
    const t0 = performance.now();
    try {
      const result = await invoke<DesignLogits>("mpnn_design_logits", {
        structureText: s.text,
        format: s.format === "pdb" ? "pdb" : "cif",
        model: this.model === "auto" ? null : this.model,
      });
      if (id !== this.requestId) return; // a newer structure or model superseded this run
      this.cache.set(cacheKey, result);
      this.data = result;
      this.elapsedMs = performance.now() - t0;
    } catch (e) {
      if (id === this.requestId) this.error = errorMessage(e);
    } finally {
      if (id === this.requestId) this.loading = false;
    }
  }
}

export const design = new DesignStore();
