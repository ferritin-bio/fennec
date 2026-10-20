<script lang="ts">
    import { onMount } from "svelte";
    import type { FennecViewer } from "#lib/structure/molstar.js";
    import { structure } from "#lib/structure/store.svelte.js";
    import { design } from "#lib/design/store.svelte.js";
    import { theme, tokenHex } from "#lib/theme.svelte.js";
    import HoverCard from "./HoverCard.svelte";

    let host: HTMLDivElement;
    let viewer = $state<FennecViewer | null>(null);
    let initError = $state<string | null>(null);
    let loadedText: string | null = null;
    let pointer = $state({ x: 0, y: 0, w: 0, h: 0, inside: false });

    onMount(() => {
        let disposed = false;
        let v: FennecViewer | undefined;
        // Mol* is large: load it only when the workspace is opened.
        import("#lib/structure/molstar.js")
            .then(({ FennecViewer }) => FennecViewer.create(host))
            .then((created) => {
                if (disposed) return created.dispose();
                v = created;
                v.onHover((r) => structure.setHover(r, "viewer"));
                v.onClick((r) => {
                    if (r) structure.selected = r;
                });
                viewer = v;
            })
            .catch((e) => (initError = e instanceof Error ? e.message : String(e)));
        return () => {
            disposed = true;
            v?.dispose();
        };
    });

    // Load the structure whenever the shared store changes.
    $effect(() => {
        const s = structure.current;
        if (!viewer || !s || s.text === loadedText) return;
        loadedText = s.text;
        viewer.load(s.text, s.format, s.id).then(() => applyColors(design.colorValues));
    });

    // Repaint by the active metric. Theme updates are serialised: only the latest values are applied.
    let painting = false;
    let pending: Map<string, number> | null = null;
    async function applyColors(values: Map<string, number>) {
        if (!viewer) return;
        pending = values;
        if (painting) return;
        painting = true;
        try {
            while (pending) {
                const next = pending;
                pending = null;
                if (next.size) await viewer.colorByValues(next);
                else await viewer.resetColors();
            }
        } finally {
            painting = false;
        }
    }

    $effect(() => {
        const values = design.colorValues;
        if (viewer && loadedText) applyColors(values);
    });

    // Highlight residues hovered in the side panels (the viewer highlights its own hovers).
    $effect(() => {
        if (!viewer || structure.hoverOrigin !== "panel") return;
        viewer.highlight(structure.hovered);
    });

    $effect(() => {
        const r = structure.selected;
        if (viewer && r) viewer.focus(r);
    });

    $effect(() => {
        theme.dark; // re-run on theme change
        if (viewer) viewer.setBackground(tokenHex("--background"));
    });

    const hoverSummary = $derived(
        structure.hoverOrigin === "viewer" ? design.summaryFor(structure.hovered) : undefined,
    );
</script>

<div
    class="relative h-full w-full overflow-hidden"
    role="application"
    aria-label="3D structure viewer"
    onpointermove={(e) => {
        const rect = e.currentTarget.getBoundingClientRect();
        pointer = { x: e.clientX - rect.left, y: e.clientY - rect.top, w: rect.width, h: rect.height, inside: true };
    }}
    onpointerleave={() => (pointer = { ...pointer, inside: false })}
>
    <div bind:this={host} class="absolute inset-0"></div>

    {#if initError}
        <div class="text-destructive absolute inset-0 grid place-items-center p-6 text-sm">{initError}</div>
    {:else if !structure.current}
        <div class="text-muted-foreground pointer-events-none absolute inset-0 grid place-items-center text-sm">
            Load a structure to begin
        </div>
    {/if}

    {#if pointer.inside && hoverSummary && design.data}
        <HoverCard summary={hoverSummary} alphabet={design.data.alphabet} {...pointer} />
    {/if}
</div>
