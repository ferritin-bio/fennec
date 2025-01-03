<script lang="ts">
    import * as Plot from "@observablehq/plot";
    import { invoke } from "@tauri-apps/api/core";

    type LogitPosition = {
        position: number; // Position in sequence
        amino_acid: string; // Index in vocabulary (0-32)
        score: number; // Logit score
    };

    type LogitPositions = LogitPosition[];

    // State + stateful ----------------------------
    const { pdbText = "" } = $props();
    let loading = $state(false);
    let error = $state(null);
    let logits = $state([]);

    $effect(() => {
        if (pdbText) fetchLogits();
    });

    async function fetchLogits() {
        if (!pdbText) return;
        loading = true;
        error = null;

        try {
            logits = await invoke("get_esm2_logits", {
                pdbSeq: pdbText,
            });
            console.log(logits);
        } catch (e) {
            error = e.message;
            console.error("Error fetching logits:", e);
        } finally {
            loading = false;
        }
    }

    function myplot(node) {
        let plot;
        function createPlot() {
            plot = Plot.plot({
                width: node.clientWidth,
                height: node.clientHeight,
                margin: 20,
                color: { type: "linear", scheme: "PiYG" },
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
                    ticks: (logits || []).reduce((acc, curr) => {
                        const pos = curr.position;
                        return pos % 10 === 0 ? [...acc, pos] : acc;
                    }, []),
                },
                marks: [
                    Plot.cell(logits || [], {
                        x: "position",
                        y: "amino_acid",
                        fill: "score",
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
    />
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
