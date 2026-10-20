<!--
  Position × amino-acid probability map for the whole structure (one column per residue),
  with the active colour metric as a strip on top. Canvas-rendered so thousands of residues stay fast.
  Hover a column to highlight that residue in 3D; click to select and focus it.
  Hover a row label to paint the structure by that amino acid.
-->
<script lang="ts">
    import { design } from "#lib/design/store.svelte.js";
    import { residueKey } from "#lib/design/logits.js";
    import { rampCss } from "#lib/design/colors.js";
    import { structure } from "#lib/structure/store.svelte.js";

    const ROW_H = 7;
    const STRIP_H = 12;
    const GAP = 4;
    const LABEL_W = 18;

    let canvas = $state<HTMLCanvasElement>();
    let scroller = $state<HTMLDivElement>();
    let containerWidth = $state(600);

    const residues = $derived(design.summaries);
    const alphabet = $derived(design.data?.alphabet ?? []);
    const cellW = $derived(residues.length ? Math.max(4, Math.min(14, (containerWidth - LABEL_W) / residues.length)) : 4);
    const width = $derived(Math.ceil(residues.length * cellW));
    const height = $derived(STRIP_H + GAP + alphabet.length * ROW_H);
    const hoveredKey = $derived(structure.hovered ? residueKey(structure.hovered) : null);
    const selectedKey = $derived(structure.selected ? residueKey(structure.selected) : null);

    $effect(() => {
        if (!canvas || !residues.length) return;
        const dpr = window.devicePixelRatio || 1;
        canvas.width = width * dpr;
        canvas.height = height * dpr;
        const ctx = canvas.getContext("2d")!;
        ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
        ctx.clearRect(0, 0, width, height);
        const values = design.colorValues;
        const wtColor = getComputedStyle(canvas).color;

        residues.forEach((r, x) => {
            const left = x * cellW;
            ctx.fillStyle = rampCss(values.get(residueKey(r)) ?? 0);
            ctx.fillRect(left, 0, cellW, STRIP_H);
            for (let a = 0; a < alphabet.length; a++) {
                ctx.fillStyle = rampCss(Math.sqrt(r.probs[a])); // sqrt lifts low-probability alternatives
                ctx.fillRect(left, STRIP_H + GAP + a * ROW_H, cellW, ROW_H);
            }
            const wt = alphabet.indexOf(r.wt);
            if (wt >= 0 && cellW >= 5) {
                ctx.fillStyle = wtColor;
                ctx.beginPath();
                ctx.arc(left + cellW / 2, STRIP_H + GAP + wt * ROW_H + ROW_H / 2, Math.min(cellW, ROW_H) / 4, 0, Math.PI * 2);
                ctx.fill();
            }
        });

        for (const [key, style] of [
            [selectedKey, "#2563eb"],
            [hoveredKey, wtColor],
        ] as const) {
            if (!key) continue;
            const x = residues.findIndex((r) => residueKey(r) === key);
            if (x < 0) continue;
            ctx.strokeStyle = style;
            ctx.lineWidth = 1.5;
            ctx.strokeRect(x * cellW + 0.75, 0.75, cellW - 1.5, height - 1.5);
        }
    });

    // Keep the hovered residue in view when it comes from the 3D viewer.
    $effect(() => {
        if (!scroller || structure.hoverOrigin !== "viewer" || !hoveredKey) return;
        const x = residues.findIndex((r) => residueKey(r) === hoveredKey) * cellW;
        if (x < scroller.scrollLeft || x > scroller.scrollLeft + scroller.clientWidth - LABEL_W) {
            scroller.scrollTo({ left: x - scroller.clientWidth / 2, behavior: "smooth" });
        }
    });

    function residueAt(e: PointerEvent | MouseEvent) {
        const x = Math.floor(e.offsetX / cellW);
        return residues[x] ?? null;
    }
</script>

{#if residues.length}
    <div class="flex text-[9px] leading-none" bind:clientWidth={containerWidth}>
        <div class="text-muted-foreground shrink-0 font-mono" style="width:{LABEL_W}px; padding-top:{STRIP_H + GAP}px">
            {#each alphabet as aa (aa)}
                <!-- svelte-ignore a11y_no_static_element_interactions -->
                <div
                    class="hover:text-foreground cursor-default"
                    style="height:{ROW_H}px"
                    onpointerenter={() => (design.previewAA = aa)}
                    onpointerleave={() => (design.previewAA = null)}
                >
                    {aa}
                </div>
            {/each}
        </div>
        <div class="min-w-0 flex-1 overflow-x-auto" bind:this={scroller}>
            <canvas
                bind:this={canvas}
                class="text-foreground block cursor-crosshair"
                style="width:{width}px; height:{height}px"
                aria-label="Per-residue amino-acid probability map"
                onpointermove={(e) => {
                    const r = residueAt(e);
                    if (r && residueKey(r) !== hoveredKey) structure.setHover(r, "panel");
                }}
                onpointerleave={() => structure.setHover(null, "panel")}
                onclick={(e) => {
                    const r = residueAt(e);
                    if (r) structure.selected = r;
                }}
            ></canvas>
        </div>
    </div>
{/if}
