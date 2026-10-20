/**
 * Types and math for per-residue design logits (ProteinMPNN / LigandMPNN).
 *
 * The backend (`mpnn_design_logits`) returns one row of log-probabilities per residue for the
 * whole structure. Temperature, softmax and summary metrics are applied here so that
 * the temperature slider and colour modes update instantly without another model call.
 */

/** Mirrors `ResidueLogits` in src-tauri/src/commands/ligandmpnn_logits.rs. */
export interface ResidueLogits {
  /** 0-based position in the model's residue order. */
  index: number;
  /** Author chain id (auth_asym_id), matching Mol* `auth_asym_id`. "" for a blank PDB chain column. */
  chain_id: string;
  /** Author residue number (auth_seq_id). */
  auth_seq_id: number;
  /** Insertion code, "" when absent. */
  ins_code: string;
  /** One-letter residue the model was fed; "X" for non-canonical residues (e.g. MSE). */
  wt: string;
  /** Log-probabilities over `DesignLogits.alphabet` (log_softmax of the raw logits; softmax(x / T) is unchanged). */
  logits: number[];
  label_asym_id: string;
  /** null for PDB input. */
  label_seq_id: number | null;
  /** Three-letter residue name from the file. */
  comp_id: string;
  /** Minimum heavy-atom distance (Å) to any ligand atom; null when the structure has no ligands. */
  ligand_distance: number | null;
}

/** A residue the backend dropped (e.g. incomplete backbone); it has no row in the matrix. */
export interface SkippedResidue {
  chain_id: string;
  auth_seq_id: number;
  ins_code: string;
  comp_id: string;
  reason: string;
}

/** Mirrors `DesignLogits` in src-tauri/src/commands/ligandmpnn_logits.rs. */
export interface DesignLogits {
  /** e.g. "protein_mpnn_v48_020" or "ligand_mpnn_v32_020_25". */
  model: string;
  /** One-letter codes in logit order; the MPNN models emit 21 (the 20 canonical + X). */
  alphabet: string[];
  residues: ResidueLogits[];
  /** Always "log_probs" today. */
  logits_kind?: string;
  uses_ligand_context?: boolean;
  n_ligand_atoms?: number;
  skipped_residues?: SkippedResidue[];
  /** Compute device the model ran on: "metal", "cuda" or "cpu". */
  device?: string;
}

/** A residue address shared by the viewer, sequence track and logits table. */
export interface ResidueKey {
  chain_id: string;
  auth_seq_id: number;
  ins_code: string;
}

export function residueKey(r: ResidueKey): string {
  return `${r.chain_id}:${r.auth_seq_id}${r.ins_code ?? ""}`;
}

/** Numerically stable softmax of `logits / temperature`. */
export function softmax(logits: ArrayLike<number>, temperature = 1): Float64Array {
  const t = Math.max(temperature, 1e-6);
  const out = new Float64Array(logits.length);
  let max = -Infinity;
  for (let i = 0; i < logits.length; i++) max = Math.max(max, logits[i] / t);
  let sum = 0;
  for (let i = 0; i < logits.length; i++) {
    out[i] = Math.exp(logits[i] / t - max);
    sum += out[i];
  }
  for (let i = 0; i < out.length; i++) out[i] /= sum;
  return out;
}

/** Shannon entropy in bits. */
export function entropyBits(p: ArrayLike<number>): number {
  let h = 0;
  for (let i = 0; i < p.length; i++) if (p[i] > 0) h -= p[i] * Math.log2(p[i]);
  return h;
}

export interface ResidueSummary extends ResidueKey {
  index: number;
  wt: string;
  probs: Float64Array;
  ligandDistance: number | null;
  /** Probability the model assigns to the wild-type residue (NaN if WT is not in the alphabet). */
  pWt: number;
  /** Most probable residue. */
  top: string;
  pTop: number;
  /** Entropy in bits; 0 = fully determined, log2(|alphabet|) = uniform. */
  entropy: number;
  /** log p(top) - log p(wt): how strongly the model would mutate this site (0 when top == wt). */
  mutationPressure: number;
}

export function summarize(data: DesignLogits, temperature: number): ResidueSummary[] {
  const { alphabet } = data;
  return data.residues.map((r) => {
    const probs = softmax(r.logits, temperature);
    let topIdx = 0;
    for (let i = 1; i < probs.length; i++) if (probs[i] > probs[topIdx]) topIdx = i;
    const wtIdx = alphabet.indexOf(r.wt);
    const pWt = wtIdx >= 0 ? probs[wtIdx] : NaN;
    return {
      index: r.index,
      chain_id: r.chain_id,
      auth_seq_id: r.auth_seq_id,
      ins_code: r.ins_code,
      wt: r.wt,
      probs,
      ligandDistance: r.ligand_distance ?? null,
      pWt,
      top: alphabet[topIdx],
      pTop: probs[topIdx],
      entropy: entropyBits(probs),
      mutationPressure: wtIdx >= 0 ? Math.log(probs[topIdx]) - Math.log(probs[wtIdx]) : 0,
    };
  });
}

/** Distance (Å) beyond which a residue counts as "not near" a ligand for colouring. */
export const LIGAND_RADIUS = 12;

export type ColorMetric = "entropy" | "pWt" | "mutationPressure" | "ligandProximity" | "substitution";

export const COLOR_METRICS: { id: ColorMetric; label: string; help: string }[] = [
  { id: "entropy", label: "Entropy", help: "How uncertain the model is at each site (bits)" },
  { id: "pWt", label: "P(wild type)", help: "Probability the model assigns to the native residue" },
  { id: "mutationPressure", label: "Mutation pressure", help: "log P(top) − log P(WT): sites the model wants to change" },
  { id: "ligandProximity", label: "Near ligand", help: "Residues within 12 Å of a ligand (LigandMPNN context)" },
  { id: "substitution", label: "P(selected AA)", help: "Probability of one chosen amino acid at every site" },
];

/**
 * Per-residue values in [0, 1] for colouring. `aa` is used by the "substitution" metric.
 * Higher always means "more interesting" (hotter colour).
 */
export function metricValues(
  summaries: ResidueSummary[],
  alphabet: string[],
  metric: ColorMetric,
  aa?: string,
): Map<string, number> {
  const out = new Map<string, number>();
  const maxEntropy = Math.log2(alphabet.length);
  const aaIdx = aa ? alphabet.indexOf(aa) : -1;
  let maxPressure = 0;
  if (metric === "mutationPressure") for (const s of summaries) maxPressure = Math.max(maxPressure, s.mutationPressure);
  for (const s of summaries) {
    let v: number;
    switch (metric) {
      case "entropy":
        v = s.entropy / maxEntropy;
        break;
      case "pWt":
        v = Number.isNaN(s.pWt) ? 0 : 1 - s.pWt;
        break;
      case "mutationPressure":
        v = maxPressure > 0 ? s.mutationPressure / maxPressure : 0;
        break;
      case "ligandProximity":
        v = s.ligandDistance === null ? 0 : 1 - s.ligandDistance / LIGAND_RADIUS;
        break;
      case "substitution":
        v = aaIdx >= 0 ? s.probs[aaIdx] : 0;
        break;
    }
    out.set(residueKey(s), Math.min(1, Math.max(0, v)));
  }
  return out;
}

/** Amino-acid physicochemical groups, used to colour substitution bars consistently everywhere. */
export const AA_GROUPS: Record<string, { group: string; color: string }> = {
  A: { group: "hydrophobic", color: "#c8a24a" },
  V: { group: "hydrophobic", color: "#c8a24a" },
  L: { group: "hydrophobic", color: "#c8a24a" },
  I: { group: "hydrophobic", color: "#c8a24a" },
  M: { group: "hydrophobic", color: "#c8a24a" },
  F: { group: "aromatic", color: "#9a6fd1" },
  W: { group: "aromatic", color: "#9a6fd1" },
  Y: { group: "aromatic", color: "#9a6fd1" },
  S: { group: "polar", color: "#3fa58a" },
  T: { group: "polar", color: "#3fa58a" },
  N: { group: "polar", color: "#3fa58a" },
  Q: { group: "polar", color: "#3fa58a" },
  D: { group: "acidic", color: "#d9534f" },
  E: { group: "acidic", color: "#d9534f" },
  K: { group: "basic", color: "#4a7fd6" },
  R: { group: "basic", color: "#4a7fd6" },
  H: { group: "basic", color: "#4a7fd6" },
  G: { group: "special", color: "#8a8f98" },
  P: { group: "special", color: "#8a8f98" },
  C: { group: "special", color: "#e0a030" },
  X: { group: "unknown", color: "#5c6370" },
};

export function aaColor(aa: string): string {
  return AA_GROUPS[aa]?.color ?? AA_GROUPS.X.color;
}
