/**
 * Applies the `.dark` class that app.css's dark tokens are keyed on, following the OS
 * colour scheme. `theme.dark` is reactive so canvases (Mol*, charts) can follow it.
 */
export const theme = $state({ dark: false });

export function initTheme() {
  if (typeof window === "undefined") return;
  const media = window.matchMedia("(prefers-color-scheme: dark)");
  const apply = () => {
    theme.dark = media.matches;
    document.documentElement.classList.toggle("dark", media.matches);
  };
  apply();
  media.addEventListener("change", apply);
}

/** Resolves a CSS colour token (e.g. "--background", stored as an HSL triplet) to #rrggbb. */
export function tokenHex(token: string): string {
  const probe = document.createElement("div");
  probe.style.color = `hsl(var(${token}))`;
  document.body.appendChild(probe);
  const rgb = getComputedStyle(probe).color;
  probe.remove();
  const m = rgb.match(/\d+(\.\d+)?/g);
  if (!m) return "#ffffff";
  return "#" + m.slice(0, 3).map((v) => Math.round(Number(v)).toString(16).padStart(2, "0")).join("");
}
