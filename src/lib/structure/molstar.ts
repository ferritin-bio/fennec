/**
 * A thin, framework-agnostic wrapper around the Mol* v6 headless plugin.
 *
 * Fennec owns every control in Svelte, so the React plugin UI is not used: we create a
 * `PluginContext` from the default spec and mount only its canvas. Residues are addressed
 * by author chain / author seq id / insertion code (`ResidueKey`), which is what the
 * Rust design commands return.
 */
import { PluginContext } from "@molstar/plugin";
import { DefaultPluginSpec } from "@molstar/plugin/default-spec";
import { PluginCommands } from "@molstar/plugin/commands";
import { StructureElement, StructureProperties, type Structure } from "@molstar/model/model/structure";
import type { Loci } from "@molstar/model/model/loci";
import { Color } from "@molstar/core/util/color";
import { ParamDefinition as PD } from "@molstar/core/util/param-definition";
import { ColorTheme } from "@molstar/graphics/theme/color";
import { ColorThemeCategory } from "@molstar/graphics/theme/color/categories";
import type { ThemeDataContext } from "@molstar/graphics/theme/theme";
import type { Location } from "@molstar/model/model/location";
import { residueKey, type ResidueKey } from "#lib/design/logits.js";
import { RAMP_HEX, UNSCORED_HEX } from "#lib/design/colors.js";

export interface PickedResidue extends ResidueKey {
  /** Three-letter residue name, e.g. "ALA". */
  comp_id: string;
  /** Mol* label_seq_id (1-based polymer index; 0 for non-polymer). */
  label_seq_id: number;
}

export type StructureFormat = "mmcif" | "pdb";

const RAMP = RAMP_HEX.map((c) => Color.fromHexStyle(c));
const UNSCORED = Color.fromHexStyle(UNSCORED_HEX);

function rampColor(t: number): Color {
  const x = Math.min(1, Math.max(0, t)) * (RAMP.length - 1);
  const i = Math.min(RAMP.length - 2, Math.floor(x));
  return Color.interpolate(RAMP[i], RAMP[i + 1], x - i);
}

const THEME_NAME = "fennec-residue-values";

const ThemeParams = {
  /** residueKey → value in [0, 1]. Hidden: set programmatically. */
  values: PD.Value<Record<string, number>>({}, { isHidden: true }),
};
type ThemeProps = PD.Values<typeof ThemeParams>;

function keyOf(loc: StructureElement.Location): string {
  return residueKey({
    chain_id: StructureProperties.chain.auth_asym_id(loc),
    auth_seq_id: StructureProperties.residue.auth_seq_id(loc),
    ins_code: StructureProperties.residue.pdbx_PDB_ins_code(loc) ?? "",
  });
}

function ResidueValueTheme(_ctx: ThemeDataContext, props: ThemeProps): ColorTheme<typeof ThemeParams> {
  const values = props.values;
  return {
    factory: ResidueValueTheme,
    granularity: "group",
    color: (location: Location) => {
      if (!StructureElement.Location.is(location)) return UNSCORED;
      const v = values[keyOf(location)];
      return v === undefined ? UNSCORED : rampColor(v);
    },
    props,
    description: "Per-residue values from Fennec's design models.",
  };
}

const ResidueValueThemeProvider: ColorTheme.Provider<typeof ThemeParams, typeof THEME_NAME> = {
  name: THEME_NAME,
  label: "Fennec residue values",
  category: ColorThemeCategory.Misc,
  factory: ResidueValueTheme,
  getParams: () => ThemeParams,
  defaultValues: PD.getDefaultValues(ThemeParams),
  isApplicable: (ctx: ThemeDataContext) => !!ctx.structure,
};

function pickResidue(loci: Loci): PickedResidue | undefined {
  if (!StructureElement.Loci.is(loci) || StructureElement.Loci.isEmpty(loci)) return undefined;
  const loc = StructureElement.Loci.getFirstLocation(loci);
  if (!loc) return undefined;
  return {
    chain_id: StructureProperties.chain.auth_asym_id(loc),
    auth_seq_id: StructureProperties.residue.auth_seq_id(loc),
    ins_code: StructureProperties.residue.pdbx_PDB_ins_code(loc) ?? "",
    comp_id: StructureProperties.atom.label_comp_id(loc),
    label_seq_id: StructureProperties.residue.label_seq_id(loc),
  };
}

export class FennecViewer {
  readonly plugin: PluginContext;
  private subs: { unsubscribe(): void }[] = [];
  private lastHoverKey: string | null = null;

  private constructor(plugin: PluginContext) {
    this.plugin = plugin;
  }

  static async create(target: HTMLElement): Promise<FennecViewer> {
    const plugin = new PluginContext(DefaultPluginSpec());
    await plugin.init();
    if (!(await plugin.mountAsync(target))) {
      plugin.dispose();
      throw new Error("Mol* could not initialise WebGL in this window.");
    }
    plugin.representation.structure.themes.colorThemeRegistry.add(ResidueValueThemeProvider);
    return new FennecViewer(plugin);
  }

  /** Hovered residue, or null when the pointer leaves all residues. Fires only on change. */
  onHover(cb: (r: PickedResidue | null) => void) {
    const sub = this.plugin.behaviors.interaction.hover.subscribe(({ current }) => {
      const r = pickResidue(current.loci) ?? null;
      const key = r ? residueKey(r) : null;
      if (key === this.lastHoverKey) return;
      this.lastHoverKey = key;
      cb(r);
    });
    this.subs.push(sub);
  }

  onClick(cb: (r: PickedResidue | null) => void) {
    this.subs.push(this.plugin.behaviors.interaction.click.subscribe(({ current }) => cb(pickResidue(current.loci) ?? null)));
  }

  async load(text: string, format: StructureFormat, label = "structure") {
    await this.plugin.clear();
    const data = await this.plugin.builders.data.rawData({ data: text, label });
    const trajectory = await this.plugin.builders.structure.parseTrajectory(data, format);
    await this.plugin.builders.structure.hierarchy.applyPreset(trajectory, "default");
  }

  private get structure(): Structure | undefined {
    return this.plugin.managers.structure.hierarchy.current.structures[0]?.cell.obj?.data;
  }

  private lociFor(r: ResidueKey): StructureElement.Loci | undefined {
    const s = this.structure;
    if (!s) return undefined;
    return StructureElement.Loci.fromSchema(s, { auth_asym_id: r.chain_id, auth_seq_id: r.auth_seq_id });
  }

  /** Paint every residue by a value in [0, 1]; residues missing from `values` are grey. */
  async colorByValues(values: Map<string, number>) {
    const obj = Object.fromEntries(values);
    for (const s of this.plugin.managers.structure.hierarchy.current.structures) {
      await this.plugin.managers.structure.component.updateRepresentationsTheme(s.components, {
        color: THEME_NAME as never,
        colorParams: { values: obj } as never,
      });
    }
  }

  async resetColors() {
    for (const s of this.plugin.managers.structure.hierarchy.current.structures) {
      await this.plugin.managers.structure.component.updateRepresentationsTheme(s.components, { color: "default" });
    }
  }

  highlight(r: ResidueKey | null) {
    const loci = r && this.lociFor(r);
    if (loci) this.plugin.managers.interactivity.lociHighlights.highlightOnly({ loci });
    else this.plugin.managers.interactivity.lociHighlights.clearHighlights();
  }

  focus(r: ResidueKey) {
    const loci = this.lociFor(r);
    if (loci) this.plugin.managers.camera.focusLoci(loci);
  }

  setBackground(hex: string) {
    PluginCommands.Canvas3D.SetSettings(this.plugin, {
      settings: (props) => {
        props.renderer.backgroundColor = Color.fromHexStyle(hex);
      },
    });
  }

  dispose() {
    for (const s of this.subs) s.unsubscribe();
    this.subs = [];
    this.plugin.dispose();
  }
}
