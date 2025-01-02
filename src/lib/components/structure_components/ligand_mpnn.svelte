<script>
    import * as Plot from "@observablehq/plot";
    import * as d3 from "d3";

    const { logits = {} } = $props();
    const plotOptions = $derived({
        margin: 20,
        style: {
            background: "transparent",
            overflow: "visible",
        },
        marks: [
            Plot.barY(logits.amino_acid_probs || [], {
                x: "amino_acid",
                y: "pseudo_prob",
                fill: "steelblue",
                rx: 2,
            }),
            Plot.ruleY([0]),
        ],
        x: {
            label: "Amino Acid",
            tickRotate: -45,
        },
        y: {
            label: "Probability",
            grid: true,
        },
    });

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
