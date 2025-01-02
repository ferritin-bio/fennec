<script>
    import * as Plot from "@observablehq/plot";
    import * as d3 from "d3";

    // export let options;
    const { logits = {}, options = {} } = $props();

    console.log("Props received:", { logits, options });

    const plotOptions = $derived({
        margin: 20,
        marks: [
            Plot.barY(logits.amino_acid_probs || [], {
                x: "amino_acid",
                y: "pseudo_prob",
            }),
        ],
    });

    console.log("Props received:", { logits, options });
    console.log("Plot options:", plotOptions);

    function myplot(node) {
        let plot;

        function createPlot() {
            const width = node.clientWidth;
            const height = node.clientHeight;

            const plotOptions = {
                ...plotOptions,
                width,
                height,
                style: {
                    width: "100%",
                    height: "100%",
                    overflow: "visible",
                },
            };

            plot = Plot.plot(plotOptions);
            node.innerHTML = "";
            node.appendChild(plot);
        }

        // Initial creation
        createPlot();

        // Handle resize
        const resizeObserver = new ResizeObserver(() => {
            createPlot();
        });
        resizeObserver.observe(node);

        return {
            destroy() {
                resizeObserver.disconnect();
                node.innerHTML = "";
            },
        };
    }
</script>

<div class="plot-container">
    Plot Options: {JSON.stringify(plotOptions)}
    <br />
    Debug: {JSON.stringify(logits)}
</div>

<!-- {#key plotOptions}
    <div
        use:myplot
        style="width: 100%; height: 100%; min-height: 400px;"
        class="plot-container"
    ></div>
{/key} -->

<style>
    .plot-container {
        width: 100%;
        height: 100%;
        max-height: 450px;
        position: relative;
    }
</style>
