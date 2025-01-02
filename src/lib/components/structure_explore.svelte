<script lang="ts">
    import { onMount } from "svelte";
    import Miew from "miew";
    import * as miewcss from "miew/dist/Miew.css";
    import { Input } from "$lib/components/ui/input/index.js";
    import { Button } from "$lib/components/ui/button/index.js";
    import { Slider } from "$lib/components/ui/slider/index.js";
    import LigBarChart from "./structure_components/ligand_mpnn.svelte";

    // Stateful Variables
    let pdbCode = $state("");
    let loading = $state(false);
    let error = $state("");
    let pdb_text = $state("");
    let current = $state({
        residue_index: 0,
        residue_name: "",
        chain: 0,
    });
    let temperature = $state(0.1);

    onMount(() => {
        const viewer = new Miew({});
        if (viewer.init()) viewer.run();

        viewer.addEventListener("newpick", (e) => {
            if (!e.obj?.residue) return;

            const residue = e.obj.residue;
            current = {
                residue_index: residue._sequence,
                residue_name: residue._type?._name,
                chain: residue._chain?._name,
            };
        });

        window.miew_viewer = viewer;
    });

    const handleSubmit = async (event: Event) => {
        event.preventDefault();
        if (!pdbCode) return;
        current = { residue_index: 0, residue_name: "", chain: 0 };
        loading = true;
        error = "";

        try {
            const response = await fetch(
                `https://files.rcsb.org/download/${pdbCode.toUpperCase()}.cif`,
            );
            if (!response.ok)
                throw new Error(`Failed to fetch: ${response.statusText}`);

            pdb_text = await response.text();
            window.miew_viewer.load(pdb_text, {
                format: "cif",
                sourceType: "immediate",
            });
        } catch (err) {
            error =
                "Failed to load structure. Please check the PDB code and try again.";
            console.error("Error:", err);
        } finally {
            loading = false;
        }
    };
</script>

<div class="flex flex-col h-screen">
    <header class="flex h-16 shrink-0 items-center gap-2 border-b px-4">
        <form
            class="flex w-full max-w-sm items-center space-x-2"
            onsubmit={handleSubmit}
        >
            <Input
                type="text"
                placeholder="PDBCode"
                bind:value={pdbCode}
                disabled={loading}
            />
            <Button type="submit" disabled={loading}>
                {loading ? "Loading..." : "Retrieve"}
            </Button>
        </form>

        <div
            class="flex flex-row items-center space-x-4 p-2 bg-gray-100 rounded-md"
        >
            {#if pdbCode}
                <div class="info-item">
                    PDB: <span class="value">{pdbCode}</span>
                </div>
            {/if}
            {#each Object.entries(current).filter(([_, v]) => v) as [key, value]}
                <div class="info-item">
                    {key.replace("_", " ")}: <span class="value">{value}</span>
                </div>
            {/each}
        </div>
    </header>

    {#if error}
        <p class="text-red-500 mt-2">{error}</p>
    {/if}

    <div class="flex flex-row w-full h-full my-4">
        <div class="w-1/2">
            <h2 class="text-2xl font-bold mb-4">Structure</h2>
            <div id="miew" class="miew-container w-full"></div>
        </div>

        <div class="w-1/2 p-4">
            <h2 class="text-2xl font-bold mb-4">LigMPNN Predictions</h2>
            {#if pdb_text}
                <div class="slider-container flex items-center gap-4">
                    <span class="font-medium min-w-20 mr-8">Temperature:</span>
                    <Slider
                        class="flex-grow"
                        type="single"
                        bind:value={temperature}
                        max={1}
                        min={0.05}
                        step={0.05}
                    />
                    <span class="temperature-value min-w-16 text-right">
                        {temperature.toFixed(2)}
                    </span>
                </div>
            {/if}

            <div class="w-4/5 h-[600px] mx-auto">
                <LigBarChart
                    pdbText={pdb_text}
                    position={current.residue_index}
                    {temperature}
                />
            </div>
        </div>
    </div>
</div>

<style>
    #miew {
        left: 10px;
        top: 10px;
        max-width: 450px;
        max-height: 450px;
    }
    .slider-container {
        @apply bg-gray-100 p-4 rounded-lg mb-4;
    }
    .slider-header {
        @apply flex justify-between items-center mb-2 font-medium;
    }
    .temperature-value {
        @apply text-gray-600 font-mono;
    }
    .info-item {
        @apply flex items-center;
    }
    .value {
        @apply text-red-500 ml-1;
    }
</style>
