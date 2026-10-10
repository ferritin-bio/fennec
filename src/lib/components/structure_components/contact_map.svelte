<script lang="ts">
    import * as Plot from "@observablehq/plot";
    import { invoke } from "@tauri-apps/api/core";
    import type { ContactMap } from "../../types";
    import { errorMessage } from "../../utils";

    // State + stateful ----------------------------
    const { pdbText = "" } = $props();
    let loading = $state(false);
    let error = $state<string | null>(null);
    let logits = $state<ContactMap[]>([]);

    $effect(() => {
        if (pdbText) fetchLogits();
    });

    async function fetchLogits() {
        if (!pdbText) return;
        loading = true;
        error = null;

        try {
            logits = await invoke<ContactMap[]>("get_amplify_contact_map", {
                pdbText: pdbText,
            });
            console.log(logits);
        } catch (e) {
            error = errorMessage(e);
            console.error("Error fetching logits:", e);
        } finally {
            loading = false;
        }
    }

    function myplot(node: HTMLElement) {
        function createPlot() {
            const plot = Plot.plot({
                width: node.clientWidth,
                height: node.clientHeight,
                margin: 20,
                // color: { type: "linear", scheme: "YlGn" },
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
                    ticks: (logits || []).reduce<number[]>((acc, curr) => {
                        const pos = curr.position_1;
                        return pos % 10 === 0 ? [...acc, pos] : acc;
                    }, []),
                },
                marks: [
                    Plot.cell(logits || [], {
                        x: "position_1",
                        y: "position_2",
                        fill: "contact_estimate",
                    }),
                ],
            });

            node.innerHTML = "";
            node.appendChild(plot);
        }

        createPlot();
        return {
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
        use:myplot
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
