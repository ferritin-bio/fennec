<script lang="ts">
    import { onMount } from "svelte";
    import Miew from "miew";
    import * as miewcss from "miew/dist/Miew.css";
    import { Input } from "$lib/components/ui/input/index.js";
    import { Button } from "$lib/components/ui/button/index.js";
    import { Slider } from "$lib/components/ui/slider/index.js";

    import { invoke } from "@tauri-apps/api/core";
    import LigBarChart from "./structure_components/ligand_mpnn.svelte";
    import * as Plot from "@observablehq/plot";

    // Stateful Variables
    let pdbCode = $state("");
    let loading = $state(false);
    let error = $state("");
    let pdb_text = $state("");
    let current_residue_index = $state(0);
    let current_residue_name = $state("");
    let current_chain = $state(0);
    let ligmpnn_logits = $state({});
    let temperature = $state(0.1);

    // PLotting State
    // let plotOptions = $derived({
    //     margin: 20,
    //     marks: [
    //         Plot.barY(ligmpnn_logits.amino_acid_probs, {
    //             x: "amino_acid",
    //             y: "pseudo_prob",
    //         }),
    //     ],
    // });

    async function lig_mpnn() {
        console.log("lig_mpnn function called with:", {
            has_pdb_text: !!pdb_text,
            current_residue: current_residue_index,
        });

        if (pdb_text && current_residue_index) {
            console.log("Calling get_ligmpnn_logits with:", {
                position: current_residue_index,
                pdb_text_length: pdb_text.length,
            });

            try {
                ligmpnn_logits = await invoke("get_ligmpnn_logits", {
                    pdbText: pdb_text,
                    position: current_residue_index,
                    temp: temperature,
                });
                console.log("Received ligmpnn_logits:", ligmpnn_logits);
            } catch (error) {
                console.error("Error in lig_mpnn:", error);
            }
        }
    }

    $effect(() => {
        if (pdb_text && current_residue_index) {
            lig_mpnn();
        }
    });

    onMount(() => {
        var viewer = new Miew({});
        if (viewer.init()) {
            viewer.run();
        }
        viewer.addEventListener("newpick", function (e) {
            try {
                if (e.obj && e.obj.residue) {
                    const res_idx = e.obj.residue._sequence;
                    const res_name = e.obj.residue._type?._name;
                    const chain_idx = e.obj.residue._chain?._index;
                    const chain_name = e.obj.residue._chain?._name;
                    let res_text = `Picked Residue: idx=${res_idx}, name=${res_name}, chain=${chain_name}, chain_idx=${chain_idx}`;
                    console.log(`Click event:`, e);
                    console.log(res_text);

                    // update stateful components
                    current_residue_index = res_idx;
                    current_residue_name = res_name;
                    current_chain = chain_name;
                } else {
                    console.log(
                        "Clicked object does not contain residue information",
                    );
                }
            } catch (error) {
                console.warn("Error handling pick event:", error);
            }
        });
        window.miew_viewer = viewer;
    });

    const fetchStructure = async (pdbCode: string) => {
        const response = await fetch(
            `https://files.rcsb.org/download/${pdbCode}.cif`,
        );
        if (!response.ok) {
            throw new Error(
                `Failed to fetch structure: ${response.statusText}`,
            );
        }
        return await response.text();
    };

    const handleSubmit = async (event: Event) => {
        event.preventDefault();
        current_residue_name = "";
        current_chain = 0;
        if (!pdbCode) return;
        loading = true;
        error = "";
        try {
            const structureData = await fetchStructure(pdbCode.toUpperCase());
            pdb_text = structureData;
            window.miew_viewer.load(structureData, {
                format: "cif",
                sourceType: "immediate",
            });
        } catch (err) {
            error =
                "Failed to load structure. Please check the PDB code and try again.";
            console.error("Error loading structure:", err);
        } finally {
            loading = false;
        }
        console.log(pdb_text);
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
                Retrieve
                {loading ? "Loading..." : "Retrieve"}
            </Button>
        </form>
        <div
            class="flex flex-row items-center space-x-4 p-2 bg-gray-100 rounded-md"
        >
            {#if pdbCode}
                <div class="flex items-center">
                    <span class="text-gray-600 font-semibold mr-1">PDB:</span>
                    <span class="text-red-500">{pdbCode}</span>
                </div>
            {/if}

            {#if current_residue_name}
                <div class="flex items-center">
                    <span class="text-gray-600 font-semibold mr-1"
                        >Residue:</span
                    >
                    <span class="text-red-500">{current_residue_name}</span>
                </div>
            {/if}
            {#if current_residue_index}
                <div class="flex items-center">
                    <span class="text-gray-600 font-semibold mr-1"
                        >Residue:</span
                    >
                    <span class="text-red-500">{current_residue_index}</span>
                </div>
            {/if}
            {#if current_chain}
                <div class="flex items-center">
                    <span class="text-gray-600 font-semibold mr-1">Chain:</span>
                    <span class="text-red-500">{current_chain}</span>
                </div>
            {/if}
        </div>
    </header>

    {#if error}
        <p class="text-red-500 mt-2">{error}</p>
    {/if}

    <div class="flex flex-row w-full h-full">
        <div class="w-1/2">
            <h2 class="text-2xl font-bold mb-4">Structure</h2>
            <div id="miew" class="miew-container w-full"></div>
        </div>

        <div class="w-1/2 p-4">
            <h2 class="text-2xl font-bold mb-4">LigMPNN Predictions</h2>
            <Slider
                type="single"
                bind:value={temperature}
                max={1}
                min={0.05}
                step={0.05}
            />
            <div style="width: 80%; height: 600px; margin: 0 auto;">
                <LigBarChart
                    logits={ligmpnn_logits}
                    options={{
                        margin: 20,
                    }}
                />
            </div>
        </div>
    </div>
</div>

<style>
    #miew {
        /* position: absolute; */
        left: 10px;
        top: 10px;
        max-width: 450px;
        max-height: 450px;
    }
</style>
