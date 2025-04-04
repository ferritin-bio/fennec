<script>
    // This component embeds the MSA (Multiple Sequence Alignment) Viewer as an iframe
    let height = "600px";
</script>

<div class="msa-viewer-component p-4 border rounded-lg shadow-sm">
    <h2 class="text-xl font-semibold mb-4">
        Multiple Sequence Alignment Viewer
    </h2>

    <div class="mb-4">
        <p class="text-slate-600">
            The Multiple Sequence Alignment (MSA) Viewer allows visualization
            and analysis of aligned biological sequences (DNA, RNA, or protein).
            It is useful for identifying conserved regions, variations, and
            evolutionary relationships between sequences.
        </p>
    </div>

    <div
        class="iframe-container border rounded-md overflow-hidden"
        style={`height: ${height};`}
    >
        <iframe
            src="https://gmod.github.io/react-msaview/"
            title="MSA Viewer"
            class="w-full h-full"
            loading="lazy"
        ></iframe>
    </div>

    <div class="mt-4">
        <label for="height-control" class="block mb-2 font-medium"
            >Height Control:</label
        >
        <input
            id="height-control"
            type="range"
            min="400"
            max="800"
            step="50"
            bind:value={height}
            class="w-full"
        />
        <div class="text-right text-sm text-slate-500">
            Current height: {height}px
        </div>
    </div>

    <div class="mt-4 text-sm text-slate-600">
        <p>Tips for using the MSA Viewer:</p>
        <ul class="list-disc ml-5 mt-2">
            <li>Click "Demo" on the embedded page to load example sequences</li>
            <li>
                Use the checkboxes in the top menu to toggle different display
                options
            </li>
            <li>Hover over sequences to see additional information</li>
            <li>Click and drag to select regions of interest</li>
        </ul>
    </div>
</div>

<style>
    /* Ensure iframe takes full width/height of container */
    iframe {
        border: none;
    }
</style>
