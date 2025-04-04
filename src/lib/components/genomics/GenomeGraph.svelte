<script>
  // This component embeds the GraphGenomeViewer as an iframe
  let height = "600px";
</script>

<div class="genome-graph-component p-4 border rounded-lg shadow-sm">
  <h2 class="text-xl font-semibold mb-4">Genome Graph Viewer</h2>
  
  <div class="mb-4">
    <p class="text-slate-600">
      The Genome Graph Viewer allows visualization of genomic data in graph format, 
      which is useful for representing structural variants, alternative haplotypes, 
      and other complex genomic relationships.
    </p>
  </div>
  
  <div class="iframe-container border rounded-md overflow-hidden" style={`height: ${height};`}>
    <iframe 
      src="https://cmdcolin.github.io/graphgenomeviewer"
      title="Graph Genome Viewer"
      class="w-full h-full"
      loading="lazy"
    ></iframe>
  </div>
  
  <div class="mt-4">
    <label for="height-control" class="block mb-2 font-medium">Height Control:</label>
    <input 
      id="height-control"
      type="range" 
      min="400" 
      max="800" 
      step="50" 
      bind:value={height} 
      class="w-full"
    />
    <div class="text-right text-sm text-slate-500">Current height: {height}px</div>
  </div>
</div>

<style>
  /* Ensure iframe takes full width/height of container */
  iframe {
    border: none;
  }
</style>