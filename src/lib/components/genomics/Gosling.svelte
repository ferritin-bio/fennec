<script>
  import { onMount } from 'svelte';
  
  let container;
  let width = "100%";
  let height = 300;
  let dataUrl = "https://server.gosling-lang.org/api/v1/tileset_info/?d=cistrome-multivec";
  let loading = true;
  
  // Load Gosling JS dynamically to avoid SSR issues
  onMount(async () => {
    try {
      // Add HiGlass CSS
      const link = document.createElement('link');
      link.rel = 'stylesheet';
      link.href = 'https://unpkg.com/higlass@1.13/dist/hglib.css';
      document.head.appendChild(link);
      
      // Load React, ReactDOM and Gosling dependencies
      const [gosling] = await Promise.all([
        import('https://esm.sh/gosling.js@0.11.0?deps=react@18,react-dom@18')
      ]);
      
      // Initialize the visualization
      gosling.embed(container, {
        tracks: [
          {
            data: {
              url: dataUrl,
              type: "multivec",
              row: "sample",
              column: "position",
              value: "peak",
              categories: ["sample 1"],
            },
            mark: "rect",
            x: { field: "position", type: "genomic" },
            color: { field: "peak", type: "quantitative", legend: true },
            width: 600,
            height: 130,
          },
        ],
      });
      
      loading = false;
    } catch (error) {
      console.error('Error loading Gosling:', error);
      loading = false;
    }
  });
</script>

<div class="gosling-component p-4 border rounded-lg shadow-sm">
  <h2 class="text-xl font-semibold mb-4">Gosling Genomics Visualization</h2>
  
  <div class="mb-4">
    <p class="text-slate-600">
      Gosling.js is a declarative grammar for interactive genomic visualization, 
      allowing detailed and customizable views of genomic data.
    </p>
  </div>
  
  {#if loading}
    <div class="flex justify-center items-center py-12">
      <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500"></div>
    </div>
  {/if}
  
  <div 
    id="gosling-container" 
    bind:this={container} 
    class="border rounded-md overflow-hidden bg-white"
    style="min-height: {height}px; width: {width};"
  ></div>
  
  <div class="text-sm text-slate-500 mt-2">
    <p>Note: This visualization uses a sample multivec dataset from the Gosling server.</p>
  </div>
</div>

<style>
  /* Add component-specific styles */
  #gosling-container {
    min-height: 300px;
  }
</style>