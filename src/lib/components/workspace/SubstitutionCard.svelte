<!--
  Substitution distribution for the hovered (or selected) residue.
  Hovering a bar repaints the whole structure by P(that amino acid) at every site,
  so you can see where else the model would accept the same replacement; clicking pins it.
-->
<script lang="ts">
    import { aaColor, AA_GROUPS } from "#lib/design/logits.js";
    import { design } from "#lib/design/store.svelte.js";
    import { structure } from "#lib/structure/store.svelte.js";

    let sortByProb = $state(true);

    const residue = $derived(structure.hovered ?? structure.selected);
    const summary = $derived(design.summaryFor(residue));
    const isPinnedSelection = $derived(!structure.hovered && !!structure.selected);

    const bars = $derived.by(() => {
        if (!summary || !design.data) return [];
        const rows = design.data.alphabet.map((aa, i) => ({ aa, p: summary.probs[i] }));
        return sortByProb ? rows.sort((a, b) => b.p - a.p) : rows;
    });
    const maxP = $derived(Math.max(1e-6, ...bars.map((b) => b.p)));

    function pin(aa: string) {
        design.focusAA = aa;
        design.metric = "substitution";
        design.previewAA = null;
    }
</script>

<section class="flex min-h-0 flex-col gap-2">
    <header class="flex items-center justify-between">
        <h3 class="text-sm font-semibold">
            Substitutions
            {#if summary}
                <span class="text-muted-foreground ml-1 font-mono font-normal">
                    {summary.chain_id}:{summary.wt}{summary.auth_seq_id}{summary.ins_code}
                </span>
                {#if isPinnedSelection}<span class="text-muted-foreground ml-1 text-xs font-normal">(selected)</span>{/if}
            {/if}
        </h3>
        <button
            class="text-muted-foreground hover:text-foreground text-xs"
            onclick={() => (sortByProb = !sortByProb)}
            aria-pressed={sortByProb}
        >
            {sortByProb ? "Sorted by P" : "Alphabetical"}
        </button>
    </header>

    {#if !design.data}
        <p class="text-muted-foreground text-sm">Run a design model to see substitution densities.</p>
    {:else if !summary}
        <p class="text-muted-foreground text-sm">
            Hover a residue in the structure or the density map. Click to pin it.
        </p>
    {:else}
        <dl class="grid grid-cols-3 gap-2 text-xs">
            <div class="bg-muted/60 rounded-md p-2">
                <dt class="text-muted-foreground">Entropy</dt>
                <dd class="font-mono text-sm">{summary.entropy.toFixed(2)} bits</dd>
            </div>
            <div class="bg-muted/60 rounded-md p-2">
                <dt class="text-muted-foreground">P(WT {summary.wt})</dt>
                <dd class="font-mono text-sm">{(summary.pWt * 100).toFixed(1)}%</dd>
            </div>
            <div class="bg-muted/60 rounded-md p-2">
                <dt class="text-muted-foreground">Top</dt>
                <dd class="font-mono text-sm">{summary.top} · {(summary.pTop * 100).toFixed(1)}%</dd>
            </div>
        </dl>
        {#if summary.ligandDistance !== null}
            <p class="text-muted-foreground -mt-1 text-xs">
                {summary.ligandDistance.toFixed(1)} Å from the nearest ligand atom
            </p>
        {/if}

        <ul class="flex flex-col gap-0.5" aria-label="Amino acid probabilities" onpointerleave={() => (design.previewAA = null)}>
            {#each bars as bar (bar.aa)}
                <li>
                    <button
                        class="hover:bg-accent group grid w-full grid-cols-[1.25rem_1fr_3.25rem] items-center gap-2 rounded px-1 py-px text-left"
                        class:bg-accent={design.metric === "substitution" && design.focusAA === bar.aa}
                        onpointerenter={() => (design.previewAA = bar.aa)}
                        onfocus={() => (design.previewAA = bar.aa)}
                        onblur={() => (design.previewAA = null)}
                        onclick={() => pin(bar.aa)}
                        title="{AA_GROUPS[bar.aa]?.group ?? ''}: paint every site by P({bar.aa})"
                    >
                        <span class="font-mono text-xs" class:font-bold={bar.aa === summary.wt}>{bar.aa}</span>
                        <span class="bg-muted relative h-3 overflow-hidden rounded-sm">
                            <span
                                class="absolute inset-y-0 left-0 rounded-sm transition-[width] duration-150"
                                style="width:{(bar.p / maxP) * 100}%; background:{aaColor(bar.aa)}"
                            ></span>
                            {#if bar.aa === summary.wt}
                                <span class="border-foreground absolute inset-0 rounded-sm border"></span>
                            {/if}
                        </span>
                        <span class="text-muted-foreground text-right font-mono text-xs tabular-nums">
                            {(bar.p * 100).toFixed(1)}%
                        </span>
                    </button>
                </li>
            {/each}
        </ul>
        <p class="text-muted-foreground text-[11px]">
            Hover a bar to paint the structure by that residue's probability at every site; click to keep it.
        </p>
    {/if}
</section>
