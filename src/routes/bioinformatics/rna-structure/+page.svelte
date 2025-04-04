<script>
  import RnaVisualizer from "$lib/components/genomics/RnaVisualizer.svelte";
</script>

<div class="container mx-auto p-4">
  <h1 class="text-2xl font-bold mb-6">RNA Secondary Structure Visualization</h1>
  
  <p class="mb-4 text-slate-600">
    This tool visualizes RNA secondary structures using RNApkin. 
    Enter an RNA sequence with its dot-bracket structure notation or load the example to get started.
  </p>
  
  <RnaVisualizer />
</div>