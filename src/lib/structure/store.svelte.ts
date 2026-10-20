/**
 * The structure currently loaded in the app, shared by every page, plus the residue
 * the user is hovering or has selected. Loading from RCSB / AlphaFold DB / disk lives
 * here so pages no longer duplicate the fetch form.
 */
import type { ResidueKey } from "#lib/design/logits.js";
import type { StructureFormat } from "./molstar.js";

export type StructureSourceKind = "pdb" | "afdb" | "file";

export interface LoadedStructure {
  /** Display id: PDB code, UniProt accession, or file name. */
  id: string;
  source: StructureSourceKind;
  format: StructureFormat;
  text: string;
}

export interface SelectedResidue extends ResidueKey {
  comp_id?: string;
}

class StructureStore {
  current = $state<LoadedStructure | null>(null);
  loading = $state(false);
  error = $state<string | null>(null);
  hovered = $state<SelectedResidue | null>(null);
  selected = $state<SelectedResidue | null>(null);
  /** Where the hover came from: the viewer already highlights its own hovers. */
  hoverOrigin = $state<"viewer" | "panel">("viewer");

  setHover(r: SelectedResidue | null, origin: "viewer" | "panel") {
    this.hoverOrigin = origin;
    this.hovered = r;
  }

  async loadPdb(code: string) {
    const id = code.trim().toUpperCase();
    if (!/^[0-9][A-Z0-9]{3}$/.test(id)) {
      this.error = `"${code}" is not a 4-character PDB id.`;
      return;
    }
    await this.fetchText(`https://files.rcsb.org/download/${id}.cif`, { id, source: "pdb", format: "mmcif" });
  }

  async loadAlphaFold(accession: string) {
    const id = accession.trim().toUpperCase();
    // The AFDB prediction API returns the current model version's file URLs.
    await this.run(async () => {
      const meta = await fetch(`https://alphafold.ebi.ac.uk/api/prediction/${id}`);
      if (!meta.ok) throw new Error(`No AlphaFold DB entry for ${id}.`);
      const [entry] = (await meta.json()) as { cifUrl: string }[];
      const res = await fetch(entry.cifUrl);
      if (!res.ok) throw new Error(`Failed to download ${entry.cifUrl}: ${res.status}`);
      return { id, source: "afdb" as const, format: "mmcif" as const, text: await res.text() };
    });
  }

  async loadFile(file: File) {
    await this.run(async () => {
      const lower = file.name.toLowerCase();
      const format: StructureFormat = lower.endsWith(".pdb") || lower.endsWith(".ent") ? "pdb" : "mmcif";
      return { id: file.name, source: "file" as const, format, text: await file.text() };
    });
  }

  private fetchText(url: string, meta: Omit<LoadedStructure, "text">) {
    return this.run(async () => {
      const res = await fetch(url);
      if (!res.ok) throw new Error(`Could not fetch ${meta.id} (${res.status} ${res.statusText}).`);
      return { ...meta, text: await res.text() };
    });
  }

  private async run(load: () => Promise<LoadedStructure>) {
    this.loading = true;
    this.error = null;
    try {
      const next = await load();
      this.hovered = null;
      this.selected = null;
      this.current = next;
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e);
    } finally {
      this.loading = false;
    }
  }
}

export const structure = new StructureStore();
