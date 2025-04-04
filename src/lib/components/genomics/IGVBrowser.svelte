<script>
  import { onMount } from 'svelte';
  
  let container;
  let genome = "hg38";
  let locus = "chr8:127,736,588-127,739,371";
  let height = 400;
  let loading = true;
  
  // Sample tracks
  const sampleTracks = [
    {
      name: "1000 Genomes - HG00103",
      url: "https://s3.amazonaws.com/1000genomes/data/HG00103/alignment/HG00103.alt_bwamem_GRCh38DH.20150718.GBR.low_coverage.cram",
      indexURL: "https://s3.amazonaws.com/1000genomes/data/HG00103/alignment/HG00103.alt_bwamem_GRCh38DH.20150718.GBR.low_coverage.cram.crai",
      format: "cram"
    }
  ];
  
  // Genome options
  const genomeOptions = [
    { id: "hg38", name: "Human (GRCh38/hg38)" },
    { id: "hg19", name: "Human (GRCh37/hg19)" },
    { id: "mm10", name: "Mouse (GRCm38/mm10)" },
    { id: "dm6", name: "Fruit fly (dm6)" },
    { id: "sacCer3", name: "Yeast (sacCer3)" }
  ];
  
  // Sample loci
  const sampleLoci = [
    { locus: "chr8:127,736,588-127,739,371", description: "MYCC gene region (chr8)" },
    { locus: "chr17:7,668,402-7,687,538", description: "TP53 gene region (chr17)" },
    { locus: "chr7:55,085,725-55,275,031", description: "EGFR gene region (chr7)" }
  ];
  
  // Initialize IGV browser
  async function initializeBrowser() {
    if (!container) return;
    
    loading = true;
    
    try {
      // Dynamically import IGV
      const igvModule = await import('https://cdn.jsdelivr.net/npm/igv@2.15.5/dist/igv.esm.min.js');
      const igv = igvModule.default;
      
      // Clear the container first
      container.innerHTML = '';
      
      // Create browser with options
      const options = {
        genome: genome,
        locus: locus,
        height: height,
        tracks: sampleTracks
      };
      
      await igv.createBrowser(container, options);
      console.log("Created IGV browser");
    } catch (error) {
      console.error('Error initializing IGV browser:', error);
    } finally {
      loading = false;
    }
  }
  
  // Watch for changes to genome or locus to reinitialize browser
  $: if (container && (genome || locus)) {
    initializeBrowser();
  }
  
  onMount(() => {
    initializeBrowser();
  });
</script>

<div class="igv-browser-component p-4 border rounded-lg shadow-sm">
  <h2 class="text-xl font-semibold mb-4">Integrative Genomics Viewer (IGV)</h2>
  
  <div class="mb-4">
    <p class="text-slate-600">
      IGV is a high-performance visualization tool for interactive exploration of 
      genomic data including SNPs, gene expression, and more.
    </p>
  </div>
  
  <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mb-4">
    <div>
      <label for="genome-select" class="block mb-2 font-medium">Genome:</label>
      <select 
        id="genome-select"
        bind:value={genome}
        class="w-full p-2 border rounded-md"
      >
        {#each genomeOptions as option}
          <option value={option.id}>{option.name}</option>
        {/each}
      </select>
    </div>
    
    <div>
      <label for="locus-select" class="block mb-2 font-medium">Jump to Region:</label>
      <select 
        id="locus-select"
        bind:value={locus}
        class="w-full p-2 border rounded-md"
      >
        {#each sampleLoci as option}
          <option value={option.locus}>{option.description}</option>
        {/each}
      </select>
    </div>
  </div>
  
  <div class="mb-4">
    <label for="height-control" class="block mb-2 font-medium">Viewer Height:</label>
    <input 
      id="height-control"
      type="range" 
      min="200" 
      max="800" 
      step="50" 
      bind:value={height} 
      class="w-full"
    />
    <div class="text-right text-sm text-slate-500">Height: {height}px</div>
  </div>
  
  {#if loading}
    <div class="flex justify-center items-center py-12">
      <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500"></div>
    </div>
  {/if}
  
  <div id="igv-container" bind:this={container} class="border rounded-md overflow-hidden bg-white"></div>
  
  <div class="text-sm text-slate-500 mt-2">
    <p>Note: This viewer loads sequence data from the 1000 Genomes Project. Initial loading may take a moment.</p>
  </div>
</div>

<style>
  /* Add component-specific styles */
  #igv-container {
    min-height: 200px;
    width: 100%;
  }
</style>