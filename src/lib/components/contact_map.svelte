<script lang="ts">
    import ContactMap from "#lib/components/structure_components/contact_map.svelte";
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

    <div class="p-4">
        {#if structure.current}
            <section>
                <h2 class="mb-2 text-lg font-semibold">AMPLIFY contact map</h2>
                <div class="h-[400px] w-full"><ContactMap structureText={structure.current.text} format={structure.current.format} /></div>
            </section>
        {:else}
            <p class="text-muted-foreground text-sm">Load a structure to predict contacts from AMPLIFY attention.</p>
        {/if}
    </div>
</div>
