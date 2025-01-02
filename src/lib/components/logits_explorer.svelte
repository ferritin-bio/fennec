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

    const handleSubmit = async (event: Event) => {
        event.preventDefault();
        if (!pdbCode) return;
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
            on:submit={handleSubmit}
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
        </div>
    </header>

    {#if error}
        <p class="text-red-500 mt-2">{error}</p>
    {/if}

    <div class="flex flex-row w-full h-full my-4">
        <div class="w-1/3">
            <h2 class="text-2xl font-bold mb-4">LigMPNN Predictions</h2>
        </div>

        <div class="w-1/3">
            <h2 class="text-2xl font-bold mb-4">ESM2</h2>
        </div>

        <div class="w-1/3">
            <h2 class="text-2xl font-bold mb-4">Amplify</h2>
        </div>
    </div>
</div>

<style>
    .info-item {
        @apply flex items-center;
    }
    .value {
        @apply text-red-500 ml-1;
    }
</style>
