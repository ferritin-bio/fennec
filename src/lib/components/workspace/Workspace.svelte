<!--
  Structure design workspace: Mol* viewer on the left, model controls and the
  substitution card on the right, and the per-residue density map along the bottom.
-->
<script lang="ts">
    import MolstarViewer from "./MolstarViewer.svelte";
    import StructureSource from "./StructureSource.svelte";
    import DesignControls from "./DesignControls.svelte";
    import SubstitutionCard from "./SubstitutionCard.svelte";
    import DensityMap from "./DensityMap.svelte";
    import { structure } from "#lib/structure/store.svelte.js";
    import { design } from "#lib/design/store.svelte.js";

    // Dev-only handle for poking at the stores from the devtools console.
    if (import.meta.env.DEV) Object.assign(window, { __fennec: { structure, design } });

    // New structure → clear old results and run the model automatically.
    let lastText: string | null = null;
    $effect(() => {
        const s = structure.current;
        if (!s || s.text === lastText) return;
        lastText = s.text;
        design.reset();
        design.run(s);
    });
</script>

<div class="grid h-[calc(100svh-2.75rem)] grid-cols-[minmax(0,1fr)_22rem] grid-rows-[auto_minmax(0,1fr)_auto]">
    <div class="col-span-2 flex items-center justify-between gap-4 border-b px-4 py-2">
        <StructureSource />
        {#if structure.current}
            <span class="text-muted-foreground truncate text-xs">
                <span class="text-foreground font-mono">{structure.current.id}</span>
                · {structure.current.source === "afdb" ? "AlphaFold DB" : structure.current.source === "pdb" ? "RCSB PDB" : "local file"}
            </span>
        {/if}
    </div>

    <div class="relative min-h-0">
        <MolstarViewer />
        {#if design.loading}
            <div class="bg-background/85 absolute top-3 left-3 rounded-md border px-3 py-1.5 text-xs shadow-sm">
                Running {design.model === "ligand_mpnn" ? "LigandMPNN (CPU)" : design.model === "protein_mpnn" ? "ProteinMPNN (Metal)" : "design model"}…
                the first run of each model loads its weights.
            </div>
        {/if}
    </div>

    <aside class="flex min-h-0 flex-col gap-5 overflow-y-auto border-l p-4">
        <DesignControls />
        <SubstitutionCard />
    </aside>

    <div class="col-span-2 border-t px-4 py-2">
        {#if design.data}
            <DensityMap />
        {:else}
            <p class="text-muted-foreground py-2 text-xs">
                The per-residue density map appears here once the model has run.
            </p>
        {/if}
    </div>
</div>
