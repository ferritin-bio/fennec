/** Sequential ramp for values in [0, 1], shared by the Mol* theme, the density map and legends. */
export const RAMP_HEX = ["#f2efe9", "#f6c453", "#e0603a", "#a3214f", "#3b0f5c"];
export const UNSCORED_HEX = "#9aa0a6";
export const RAMP_CSS = `linear-gradient(to right, ${RAMP_HEX.join(", ")})`;

const RAMP_RGB = RAMP_HEX.map((h) => [1, 3, 5].map((i) => parseInt(h.slice(i, i + 2), 16)));

/** [r, g, b] for t in [0, 1]. */
export function rampRgb(t: number): [number, number, number] {
  const x = Math.min(1, Math.max(0, t)) * (RAMP_RGB.length - 1);
  const i = Math.min(RAMP_RGB.length - 2, Math.floor(x));
  const f = x - i;
  const a = RAMP_RGB[i];
  const b = RAMP_RGB[i + 1];
  return [0, 1, 2].map((k) => Math.round(a[k] + (b[k] - a[k]) * f)) as [number, number, number];
}

export function rampCss(t: number): string {
  const [r, g, b] = rampRgb(t);
  return `rgb(${r} ${g} ${b})`;
}

/** Black or white text for legibility on a ramp colour. */
export function rampInk(t: number): string {
  const [r, g, b] = rampRgb(t);
  return 0.299 * r + 0.587 * g + 0.114 * b > 150 ? "#1b1b1f" : "#ffffff";
}
