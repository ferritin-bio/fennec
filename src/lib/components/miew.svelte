<script lang="ts">
    import { onMount } from "svelte";
    import Miew from "miew";
    import * as miewcss from "miew/dist/Miew.css";
    import { Input } from "$lib/components/ui/input/index.js";
    import { Button } from "$lib/components/ui/button/index.js";
    let pdbCode = "";
    let loading = false;
    let error = "";

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
                    const pickResultElement =
                        document.getElementById("pickResult");
                    pickResultElement.textContent = res_text;
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
        if (!pdbCode) return;
        loading = true;
        error = "";
        try {
            const structureData = await fetchStructure(pdbCode.toUpperCase());
            // console.log(structureData);
            console.log(window.miew_viewer);
            window.miew_viewer.load(structureData, {
                format: "cif",
                sourceType: "immediate", // needed to load the string
            });
        } catch (err) {
            error =
                "Failed to load structure. Please check the PDB code and try again.";
            console.error("Error loading structure:", err);
        } finally {
            loading = false;
        }
    };
</script>

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

<h1 id="pickResult"></h1>

{#if error}
    <p class="text-red-500 mt-2">{error}</p>
{/if}

<div id="miew" class="miew-container"></div>

<style>
    #miew {
        /* position: absolute; */
        left: 50px;
        top: 50px;
        width: 450px;
        height: 450px;
    }
</style>
