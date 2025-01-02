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

            const finalOptions = {
                ...plotOptions,
                width,
                height,
                style: {
                    width: "100%",
                    height: "100%",
                    overflow: "visible",
                },
            };

            plot = Plot.plot(finalOptions);
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

{#key plotOptions}
    <div
        use:myplot
        style="width: 100%; height: 100%; min-height: 400px;"
        class="plot-container"
    />
{/key}

<style>
    .plot-container {
        width: 100%;
        height: 100%;
        max-height: 450px;
        position: relative;
    }
</style>
