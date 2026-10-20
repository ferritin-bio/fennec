<!-- Model choice, run button, temperature, colour metric and legend for the design workspace. -->
<script lang="ts">
    import Play from "@lucide/svelte/icons/play";
    import { Button } from "#lib/components/ui/button/index.js";
    import { Slider } from "#lib/components/ui/slider/index.js";
    import { COLOR_METRICS, LIGAND_RADIUS } from "#lib/design/logits.js";
    import { RAMP_CSS } from "#lib/design/colors.js";
    import { design, type DesignModel } from "#lib/design/store.svelte.js";
    import { structure } from "#lib/structure/store.svelte.js";

    const models: { id: DesignModel; label: string }[] = [
        { id: "auto", label: "Auto" },
        { id: "protein_mpnn", label: "ProteinMPNN" },
        { id: "ligand_mpnn", label: "LigandMPNN" },
    ];

    function modelLabel(id: string) {
        if (id.startsWith("ligand_mpnn")) return "LigandMPNN";
        if (id.startsWith("protein_mpnn")) return "ProteinMPNN";
        return id;
    }

    const metricInfo = $derived(COLOR_METRICS.find((m) => m.id === design.activeMetric)!);
    const legend = $derived.by(() => {
        switch (design.activeMetric) {
            case "entropy":
                return ["certain", "uncertain"];
            case "pWt":
                return ["WT favoured", "WT disfavoured"];
            case "mutationPressure":
                return ["keep", "mutate"];
            case "ligandProximity":
                return [`≥ ${LIGAND_RADIUS} Å`, "contact"];
            case "substitution":
                return [`P(${design.activeAA}) = 0`, `P(${design.activeAA}) = 1`];
        }
    });
</script>

<section class="flex flex-col gap-3">
    <div class="flex items-center gap-2">
        <select
            class="border-input bg-background h-8 rounded-md border px-2 text-sm"
            aria-label="Design model"
            bind:value={design.model}
            onchange={() => design.reset()}
        >
            {#each models as m (m.id)}<option value={m.id}>{m.label}</option>{/each}
        </select>
        <Button
            size="sm"
            disabled={!structure.current || design.loading}
            onclick={() => structure.current && design.run(structure.current)}
        >
            <Play class="size-4" />
            {design.loading ? "Running…" : design.data ? "Re-run" : "Run"}
        </Button>
    </div>
    {#if design.error}
        <p class="text-destructive text-xs" role="alert">{design.error}</p>
    {/if}
    {#if design.data}
        {@const d = design.data}
        {@const skipped = d.skipped_residues ?? []}
        <p class="text-muted-foreground text-xs leading-relaxed">
            <span class="text-foreground">{modelLabel(d.model)}</span>
            · {d.residues.length} residues
            {#if design.elapsedMs !== null}· {(design.elapsedMs / 1000).toFixed(1)}s{/if}
            {#if d.device}· {d.device === "metal" ? "Metal GPU" : d.device.toUpperCase()}{/if}
            {#if d.uses_ligand_context}· {d.n_ligand_atoms} ligand atoms in context{/if}
            {#if skipped.length}
                · <span
                    class="underline decoration-dotted"
                    title={skipped.map((r) => `${r.chain_id}:${r.comp_id}${r.auth_seq_id}${r.ins_code} (${r.reason})`).join("\n")}
                    >{skipped.length} skipped</span
                >
            {/if}
        </p>
    {/if}

    <label class="grid grid-cols-[6.5rem_1fr_2.5rem] items-center gap-3 text-sm">
        <span class="text-muted-foreground">Temperature</span>
        <Slider type="single" bind:value={design.temperature} min={0.05} max={2} step={0.05} aria-label="Sampling temperature" />
        <span class="text-right font-mono text-xs tabular-nums">{design.temperature.toFixed(2)}</span>
    </label>

    <div class="flex flex-col gap-1.5">
        <span class="text-muted-foreground text-sm">Colour structure by</span>
        <div class="bg-muted grid grid-cols-2 gap-0.5 rounded-md p-0.5 text-xs" role="radiogroup" aria-label="Colour metric">
            {#each COLOR_METRICS as m (m.id)}
                <button
                    role="radio"
                    aria-checked={design.metric === m.id}
                    title={m.help}
                    class="rounded px-2 py-1 {design.metric === m.id ? 'bg-background shadow-sm' : 'text-muted-foreground hover:text-foreground'}"
                    onclick={() => (design.metric = m.id)}
                >
                    {m.id === "substitution" ? `P(${design.focusAA})` : m.label}
                </button>
            {/each}
        </div>
        <div class="mt-1">
            <div class="h-2 rounded-full" style="background:{RAMP_CSS}"></div>
            <div class="text-muted-foreground mt-0.5 flex justify-between text-[11px]">
                <span>{legend[0]}</span>
                <span class="text-foreground">{metricInfo.label}{design.previewAA ? " (preview)" : ""}</span>
                <span>{legend[1]}</span>
            </div>
        </div>
    </div>
</section>
