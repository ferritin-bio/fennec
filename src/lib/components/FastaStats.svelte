<script>
    import { invoke } from "@tauri-apps/api/core";
    import { open } from "@tauri-apps/plugin-dialog";
    import { onMount } from "svelte";
    import * as Plot from "@observablehq/plot";

    let stats = null;
    let loading = false;
    let error = null;
    let plotElement;
    let plotWidth = 0;

    onMount(() => {
        const resizeObserver = new ResizeObserver((entries) => {
            for (const entry of entries) {
                if (entry.target === plotElement?.parentElement) {
                    plotWidth = entry.contentRect.width;
                    if (stats?.contig_lengths) {
                        renderPlot();
                    }
                }
            }
        });

        if (plotElement?.parentElement) {
            resizeObserver.observe(plotElement.parentElement);
            plotWidth = plotElement.parentElement.offsetWidth;
        }

        return () => {
            resizeObserver.disconnect();
        };
    });

    function renderPlot() {
        if (
            !plotElement ||
            !stats?.contig_lengths ||
            stats.contig_lengths.length === 0
        )
            return;
        // // Clear previous plot
        // while (plotElement.firstChild) {
        //     plotElement.firstChild.remove();
        // }

        // // Make sure we have valid data
        // const lengths = stats.contig_lengths.filter(
        //     (length) => typeof length === "number" && length > 0,
        // );

        // console.log(lengths);

        // Create a new histogram plot
        const plot = Plot.plot({
            width: 600,
            height: 300,
            marginLeft: 60,
            marginBottom: 40,
            x: { label: "Sequence Length" },
            y: { label: "Count" },
            style: {
                background: "transparent",
                color: "currentColor",
                fontFamily: "inherit",
            },
            marks: [
                Plot.rectY(stats.contig_lengths, Plot.binX({ y: "count" })),
            ],
        });
        plotElement.appendChild(plot);
    }

    async function getFastaStats() {
        try {
            loading = true;
            error = null;

            // Open file dialog for the user to select a FASTA file
            const fasta = await open({
                multiple: false,
                filters: [
                    {
                        name: "Fasta",
                        extensions: ["fa", "fasta", "fna"],
                    },
                ],
            });

            // User might cancel the file selection
            if (!fasta) {
                loading = false;
                return;
            }

            // Call the Rust function to analyze the FASTA file
            stats = await invoke("get_stats", { filename: fasta });
        } catch (err) {
            error = err.toString();
            console.error("Error getting FASTA stats:", err);
        } finally {
            loading = false;
        }
    }

    async function getDetailedFastaStats() {
        try {
            loading = true;
            error = null;

            const fasta = await open({
                multiple: false,
                filters: [
                    {
                        name: "Fasta",
                        extensions: ["fa", "fasta", "fna"],
                    },
                ],
            });

            if (!fasta) {
                loading = false;
                return;
            }

            // Call the more detailed stats function
            stats = await invoke("get_seqstats", { filename: fasta });

            // Render the plot when stats are loaded
            setTimeout(() => {
                if (stats?.contig_lengths) {
                    renderPlot();
                }
            }, 10);
        } catch (err) {
            error = err.toString();
            console.error("Error getting detailed FASTA stats:", err);
        } finally {
            loading = false;
        }
    }
</script>

<div class="fasta-stats-container p-4 border rounded-lg shadow-sm">
    <h2 class="text-xl font-semibold mb-4">FASTA Statistics</h2>
    <div class="flex gap-4 mb-6">
        <button
            on:click={getFastaStats}
            class="px-4 py-2 bg-blue-500 text-white rounded-md hover:bg-blue-600 focus:outline-none focus:ring-2 focus:ring-blue-300 disabled:opacity-50"
            disabled={loading}
        >
            {loading ? "Loading..." : "Basic Stats"}
        </button>
        <button
            on:click={getDetailedFastaStats}
            class="px-4 py-2 bg-green-500 text-white rounded-md hover:bg-green-600 focus:outline-none focus:ring-2 focus:ring-green-300 disabled:opacity-50"
            disabled={loading}
        >
            {loading ? "Loading..." : "Detailed Stats"}
        </button>
    </div>

    {#if error}
        <div class="p-3 bg-red-100 text-red-800 rounded-md mb-4">
            {error}
        </div>
    {/if}

    {#if stats}
        <div class="stats-results p-4 bg-gray-50 rounded-md">
            <h3 class="text-lg font-medium mb-2">Results</h3>

            {#if "recordcount" in stats}
                <!-- Basic stats -->
                <div class="grid grid-cols-2 gap-2">
                    <div class="stat-label font-medium">Record Count:</div>
                    <div>{stats.recordcount}</div>

                    <div class="stat-label font-medium">Maximum Length:</div>
                    <div>{stats.maxlength}</div>
                </div>
            {:else}
                <!-- Detailed stats -->
                <div class="grid grid-cols-2 gap-2 mb-4">
                    <div class="stat-label font-medium">File:</div>
                    <div>{stats.filename}</div>

                    <div class="stat-label font-medium">Format:</div>
                    <div>{stats.format}</div>

                    <div class="stat-label font-medium">Type:</div>
                    <div>{stats.datatype}</div>

                    <div class="stat-label font-medium">Sequences:</div>
                    <div>{stats.num_seqs}</div>

                    <div class="stat-label font-medium">Total Length:</div>
                    <div>{stats.sum_len}</div>

                    <div class="stat-label font-medium">Min Length:</div>
                    <div>{stats.min_len}</div>

                    <div class="stat-label font-medium">Average Length:</div>
                    <div>{stats.avg_len.toFixed(2)}</div>

                    <div class="stat-label font-medium">Max Length:</div>
                    <div>{stats.max_len}</div>
                </div>

                <!-- Sequence Length Histogram -->
                <div class="mt-6">
                    <h3 class="text-lg font-medium mb-2">
                        Sequence Length Distribution
                    </h3>
                    <div class="plot-container" bind:this={plotElement}></div>
                </div>
            {/if}
        </div>
    {/if}
</div>

<style>
    .plot-container {
        width: 100%;
        min-height: 400px;
        margin-top: 1rem;
        margin-bottom: 1rem;
        /* Critical fix for SVG sizing */
        display: block;
    }

    /* Fix SVG scaling */
    :global(.plot-container > svg) {
        width: 100% !important;
        height: auto !important;
        display: block !important;
    }

    /* Override any inline styles that might be causing problems */
    :global(svg.plot) {
        width: 100% !important;
        max-width: none !important;
        height: auto !important;
        max-height: none !important;
    }
</style>
