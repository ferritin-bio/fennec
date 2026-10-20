import { describe, expect, it } from "vitest";
import { entropyBits, metricValues, residueKey, softmax, summarize, type DesignLogits } from "./logits";

const extra = { label_asym_id: "A", label_seq_id: null, comp_id: "UNK", ligand_distance: null };

const data: DesignLogits = {
  model: "test",
  alphabet: ["A", "C", "D"],
  residues: [
    { ...extra, index: 0, chain_id: "A", auth_seq_id: 1, ins_code: "", wt: "A", logits: [2, 0, 0] },
    { ...extra, index: 1, chain_id: "A", auth_seq_id: 2, ins_code: "B", wt: "C", logits: [0, 0, 0] },
  ],
};

describe("softmax", () => {
  it("sums to one and is shift invariant", () => {
    const p = softmax([1, 2, 3]);
    expect(p.reduce((a, b) => a + b, 0)).toBeCloseTo(1);
    expect(Array.from(softmax([101, 102, 103]))).toEqual(Array.from(p).map((v) => expect.closeTo(v, 12)));
  });

  it("sharpens as temperature drops", () => {
    expect(softmax([1, 0], 0.1)[0]).toBeGreaterThan(softmax([1, 0], 1)[0]);
  });
});

describe("summaries", () => {
  it("computes entropy, WT probability and top residue", () => {
    const [a, b] = summarize(data, 1);
    expect(a.top).toBe("A");
    expect(a.mutationPressure).toBe(0);
    expect(b.entropy).toBeCloseTo(Math.log2(3));
    expect(b.pWt).toBeCloseTo(1 / 3);
    expect(entropyBits([1, 0, 0])).toBe(0);
  });

  it("keys residues with insertion codes", () => {
    expect(residueKey(data.residues[1])).toBe("A:2B");
    const values = metricValues(summarize(data, 1), data.alphabet, "entropy");
    expect(values.get("A:2B")).toBeCloseTo(1);
  });
});
