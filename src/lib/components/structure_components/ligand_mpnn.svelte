<script lang="ts">
    import * as Plot from "@observablehq/plot";
    import { invoke } from "@tauri-apps/api/core";
    import type { PseudoProbability } from "../../types";

    const { pdbText = "", position = 0, temperature = 0.1 } = $props();

    let loading = $state(false);
    let error = $state(null);
    let logits = $state<PseudoProbability[]>([]);

    $effect(() => {
        if (pdbText && position) fetchLogits();
    });

    async function fetchLogits() {
        if (!pdbText || !position) return;
        loading = true;
        error = null;

        try {
            const result = await invoke("get_ligmpnn_logits", {
                pdbText,
                position,
                temp: temperature,
            });
            logits = result;
        } catch (e) {
            error = typeof e === "string" ? e : (e?.message ?? String(e));
        } finally {
            loading = false;
        }
    }

    function myplot(node, data) {
        function createPlot(data) {
            const plot = Plot.plot({
                width: node.clientWidth,
                height: node.clientHeight,
                margin: 20,
                style: {
                    width: "100%",
                    height: "100%",
                    background: "transparent",
                    overflow: "visible",
                },
                marks: [
                    Plot.barY(data || [], {
                        x: "amino_acid",
                        y: "pseudo_prob",
                        fill: "orange",
                        rx: 2,
                    }),
                    Plot.ruleY([0]),
                ],
                x: {
                    label: "Amino Acid",
                    tickRotate: -45,
                    labelOffset: 35,
                },
                y: {
                    label: "Probability",
                    grid: true,
                },
            });
            node.innerHTML = "";
            node.appendChild(plot);
        }

        createPlot(data);
        return {
            update(newData) {
                createPlot(newData);
            },
            destroy() {
                node.innerHTML = "";
            },
        };
    }
</script>

{#if loading}
    <div class="loading">Loading...</div>
{:else if error}
    <div class="error">{error}</div>
{:else if !logits?.length}
    <div class="no-data">Select a residue to view LigMPNN predictions</div>
{:else}
    <div
        use:myplot={logits}
        style="width: 100%; height: 100%; min-height: 400px;"
        class="plot-container"
    ></div>
{/if}

<style>
    .plot-container {
        width: 100%;
        height: 100%;
        max-height: 450px;
        position: relative;
    }

    .no-data {
        height: 100%;
        display: flex;
        align-items: center;
        justify-content: center;
        color: #666;
        font-style: italic;
    }
</style>
