<script>
    import * as Plot from "@observablehq/plot";
    import * as d3 from "d3";
    export let options;

    function myplot(node) {
        let plot;

        function createPlot() {
            const width = node.clientWidth;
            const height = node.clientHeight;

            const plotOptions = {
                ...options,
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

{#key options}
    <div
        use:myplot
        {...$$restProps}
        style="width: 100%; height: 100%; min-height: 400px;"
        class="plot-container"
    ></div>
{/key}

<style>
    .plot-container {
        width: 100%;
        height: 100%;
        max-height: 450px;
        position: relative;
    }
</style>
