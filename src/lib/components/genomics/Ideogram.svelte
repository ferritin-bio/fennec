<script>
  import { onMount } from 'svelte';
  import Ideogram from 'ideogram';
  
  let container;
  let organism = "4932"; // Default: S. cerevisiae
  
  // List of common model organisms
  const organisms = [
    { id: "9606", name: "Human (Homo sapiens)" },
    { id: "10090", name: "Mouse (Mus musculus)" },
    { id: "7227", name: "Fruit fly (Drosophila melanogaster)" },
    { id: "4932", name: "Yeast (Saccharomyces cerevisiae)" },
    { id: "6239", name: "Nematode (Caenorhabditis elegans)" },
    { id: "7955", name: "Zebrafish (Danio rerio)" },
    { id: "3702", name: "Thale cress (Arabidopsis thaliana)" }
  ];
  
  // Initialize ideogram
  function renderIdeogram() {
    if (!container) return;
    
    // Clear previous ideogram
    container.innerHTML = '';
    
    // Create new ideogram
    new Ideogram({
      organism: organism,
      container: '#ideogram-container',
      orientation: 'horizontal'
    });
  }
  
  // Re-render when organism changes
  $: if (organism) {
    // Use setTimeout to ensure DOM is updated
    setTimeout(renderIdeogram, 0);
  }
  
  onMount(() => {
    renderIdeogram();
  });
</script>

<div class="ideogram-component p-4 border rounded-lg shadow-sm">
  <h2 class="text-xl font-semibold mb-4">Chromosome Visualization</h2>
  
  <div class="mb-4">
    <label for="organism-select" class="block mb-2 font-medium">Select Organism:</label>
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
  
  <div id="ideogram-container" bind:this={container} class="mt-4 border p-4 rounded-md bg-white min-h-[200px]"></div>
</div>

<style>
  /* Add any component-specific styling here */
  #ideogram-container {
    width: 100%;
    overflow-x: auto;
  }
</style>