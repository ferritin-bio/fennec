<script lang="ts">
    import ESM2Logits from "#lib/components/structure_components/esm2_logits.svelte";
    import AmplifyLogits from "#lib/components/structure_components/amplify_logits.svelte";
    import StructureSource from "#lib/components/workspace/StructureSource.svelte";
    import { structure } from "#lib/structure/store.svelte.js";
</script>

<div class="flex flex-col">
    <div class="flex items-center justify-between gap-4 border-b px-4 py-2">
        <StructureSource />
        {#if structure.current}
            <span class="text-muted-foreground font-mono text-xs">{structure.current.id}</span>
        {/if}
    </div>

    <div class="space-y-6 p-4">
        {#if structure.current}
            <section>
                <h2 class="mb-2 text-lg font-semibold">ESM2</h2>
                <div class="h-[400px] w-full"><ESM2Logits structureText={structure.current.text} format={structure.current.format} /></div>
            </section>
            <section>
                <h2 class="mb-2 text-lg font-semibold">AMPLIFY</h2>
                <div class="h-[400px] w-full"><AmplifyLogits structureText={structure.current.text} format={structure.current.format} /></div>
            </section>
        {:else}
            <p class="text-muted-foreground text-sm">Load a structure to compute per-position sequence logits.</p>
        {/if}
    </div>
</div>
