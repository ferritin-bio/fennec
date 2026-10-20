<!-- Floating per-residue substitution density shown next to the pointer while hovering the structure. -->
<script lang="ts">
    import { aaColor, type ResidueSummary } from "#lib/design/logits.js";

    const {
        summary,
        alphabet,
        x,
        y,
        w,
        h,
    }: { summary: ResidueSummary; alphabet: string[]; x: number; y: number; w: number; h: number } = $props();

    const CARD_W = 248;
    const CARD_H = 150;
    // Prefer the right of the pointer, flip left near the edge, and always stay inside the viewer.
    const left = $derived(Math.max(8, Math.min(x + 16 + CARD_W > w ? x - 16 - CARD_W : x + 16, w - CARD_W - 8)));
    const top = $derived(Math.max(8, Math.min(y + 16, h - CARD_H - 8)));
    const maxP = $derived(Math.max(...summary.probs));
</script>

<div
    class="bg-popover/95 text-popover-foreground pointer-events-none absolute z-20 rounded-lg border p-2.5 shadow-lg backdrop-blur"
    style="left:{left}px; top:{top}px; width:{CARD_W}px"
>
    <div class="mb-1.5 flex items-baseline justify-between text-xs">
        <span class="font-mono font-semibold">{summary.chain_id}:{summary.wt}{summary.auth_seq_id}{summary.ins_code}</span>
        <span class="text-muted-foreground">
            H {summary.entropy.toFixed(2)} bits · P(WT) {(summary.pWt * 100).toFixed(0)}%{summary.ligandDistance !== null
                ? ` · ${summary.ligandDistance.toFixed(1)} Å lig`
                : ""}
        </span>
    </div>
    <div class="flex h-16 items-end gap-px">
        {#each alphabet as aa, i (aa)}
            <div
                class="flex-1 rounded-t-[2px]"
                class:wt={aa === summary.wt}
                style="height:{(summary.probs[i] / maxP) * 100}%; background:{aaColor(aa)}"
            ></div>
        {/each}
    </div>
    <div class="text-muted-foreground mt-0.5 flex gap-px font-mono text-[9px] leading-none">
        {#each alphabet as aa (aa)}
            <span class="flex-1 text-center" class:text-foreground={aa === summary.wt} class:font-bold={aa === summary.wt}>{aa}</span>
        {/each}
    </div>
    <div class="text-muted-foreground mt-1.5 text-[11px]">
        {#if summary.top === summary.wt}
            Model agrees with wild type ({(summary.pTop * 100).toFixed(0)}%)
        {:else}
            Prefers <span class="text-foreground font-semibold">{summary.wt}{summary.auth_seq_id}{summary.top}</span>
            ({(summary.pTop * 100).toFixed(0)}% vs {(summary.pWt * 100).toFixed(0)}%)
        {/if}
    </div>
</div>

<style>
    .wt {
        outline: 1.5px solid var(--color-foreground);
        outline-offset: 0;
    }
</style>
