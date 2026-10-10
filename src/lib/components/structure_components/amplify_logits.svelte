<script lang="ts">
    import * as Plot from "@observablehq/plot";
    import { invoke } from "@tauri-apps/api/core";
    import type { PseudoProbability } from "../../types";
    import { errorMessage } from "../../utils";

    // State + stateful ----------------------------
    const { pdbText = "" }: { pdbText?: string } = $props();
    let loading = $state(false);
    let error = $state<string | null>(null);
    let logits = $state<PseudoProbability[]>([]);

    $effect(() => {
        if (pdbText) fetchLogits();
    });

    async function fetchLogits() {
        if (!pdbText) return;
        loading = true;
        error = null;

        try {
            const result = await invoke<PseudoProbability[]>("get_amplify_logits", {
                pdbText: pdbText,
            });
            logits = result;
        } catch (e) {
            error = errorMessage(e);
        } finally {
            loading = false;
        }
    }

    function myplot(node: HTMLElement, data: PseudoProbability[]) {
        function createPlot(data: PseudoProbability[]) {
            const plot = Plot.plot({
                width: node.clientWidth,
                height: node.clientHeight,
                margin: 20,
                color: { type: "linear", scheme: "YlGn" },
                style: {
                    width: "100%",
                    height: "100%",
                    background: "transparent",
                    overflow: "visible",
                },
                y: {
                    label: "Amino Acid →",
                    labelOffset: 30,
                },
                x: {
                    tickFormat: "",
                    labelOffset: 30,
                    ticks: (data || []).reduce<number[]>((acc, curr) => {
                        const pos = curr.position;
                        return pos % 10 === 0 ? [...acc, pos] : acc;
                    }, []),
                },
                marks: [
                    Plot.cell(data || [], {
                        x: "position",
                        y: "amino_acid",
                        fill: "pseudo_prob",
                    }),
                ],
            });

            node.innerHTML = "";
            node.appendChild(plot);
        }

        createPlot(data);
        return {
            update(newData: PseudoProbability[]) {
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
        /* height: 100%; */
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
