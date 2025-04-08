<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import Ideogram from "ideogram";

    let container;
    let organism = "4932"; // Default: S. cerevisiae
    let ideogramInstance = null;
    let isRendering = false;
    let isLoading = true;
    let errorMessage = null;

    // List of common model organisms
    const organisms = [
        { id: "9606", name: "Human (Homo sapiens)" },
        { id: "10090", name: "Mouse (Mus musculus)" },
        { id: "7227", name: "Fruit fly (Drosophila melanogaster)" },
        { id: "4932", name: "Yeast (Saccharomyces cerevisiae)" },
        { id: "6239", name: "Nematode (Caenorhabditis elegans)" },
        { id: "7955", name: "Zebrafish (Danio rerio)" },
        { id: "3702", name: "Thale cress (Arabidopsis thaliana)" },
    ];

    async function renderIdeogram() {
        if (!container) return;

        // Reset state
        isLoading = true;
        errorMessage = null;

        // Clear previous ideogram instance and DOM
        if (ideogramInstance) {
            try {
                if (typeof ideogramInstance.destroy === "function") {
                    ideogramInstance.destroy();
                }
            } catch (e) {
                console.warn("Error cleaning up previous ideogram:", e);
            }
            ideogramInstance = null;
        }

        // Clear container
        container.innerHTML = "";

        try {
            // Create new ideogram with proper configuration and error handling
            ideogramInstance = new Ideogram({
                organism: organism,
                container: "#ideogram-container",
                orientation: "horizontal",
                chromosomeWidth: 12,
                chrHeight: 400,
                chrMargin: 10,
                showBandLabels: true,
                showFullyBanded: true,
                // Add callback for when drawing is complete
                onLoad: function () {
                    isLoading = false;
                },
                // Handle drawing errors
                onError: function (error) {
                    console.error("Ideogram error:", error);
                    errorMessage = "Failed to load chromosome data: " + error;
                    isLoading = false;
                },
            });
        } catch (error) {
            console.error("Error initializing ideogram:", error);
            errorMessage =
                "Failed to initialize chromosome visualization: " +
                error.message;
            isLoading = false;
        }
    }

    // Re-render when organism changes
    $: if (organism && container && !isRendering) {
        // Use setTimeout to ensure DOM is updated
        setTimeout(renderIdeogram, 10);
    }

    onMount(() => {
        renderIdeogram();
    });

    onDestroy(() => {
        // Clean up when component is destroyed
        if (
            ideogramInstance &&
            typeof ideogramInstance.destroy === "function"
        ) {
            ideogramInstance.destroy();
        }
    });
</script>

<div class="ideogram-component p-4 border rounded-lg shadow-sm">
    <h2 class="text-xl font-semibold mb-4">Chromosome Visualization</h2>

    <div class="mb-4">
        <label for="organism-select" class="block mb-2 font-medium"
            >Select Organism:</label
        >
        <select
            id="organism-select"
            bind:value={organism}
            class="w-full p-2 border rounded-md"
        >
            {#each organisms as org}
                <option value={org.id}>{org.name}</option>
            {/each}
        </select>
    </div>

    <div
        id="ideogram-container"
        bind:this={container}
        class="mt-4 border p-4 rounded-md bg-white min-h-[200px]"
    ></div>
</div>

<style>
    /* Add any component-specific styling here */
    #ideogram-container {
        width: 100%;
        overflow-x: auto;
    }
</style>
