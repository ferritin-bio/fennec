<script>
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  
  let rnaInput = '';
  let height = 400;
  let angle = 0;
  let bubbleRadius = 0.5;
  let mirrorX = false;
  let mirrorY = false;
  let colorTheme = "default";
  let svgOutput = '';
  let loading = false;
  let error = null;
  let svgContainer;

  async function visualizeRNA() {
    if (!rnaInput.trim()) {
      error = "Please enter RNA sequence with structure";
      return;
    }
    
    try {
      loading = true;
      error = null;
      
      // Call the Rust function to visualize RNA structure
      svgOutput = await invoke("rnapkin_fn", {
        sequence: rnaInput,
        height,
        color_theme: colorTheme,
        mirror_x: mirrorX,
        mirror_y: mirrorY,
        rotation_angle: angle,
        bubble_radius: bubbleRadius
      });
      
      // Insert SVG into the container
      if (svgContainer) {
        svgContainer.innerHTML = svgOutput;
      }
      
    } catch (err) {
      error = err.toString();
      console.error("Error visualizing RNA:", err);
    } finally {
      loading = false;
    }
  }

  // Example RNA sequence with structure
  const exampleRNA = `>fantastic guanine riboswitch
AAUAUAAUAGGAACACUCAUAUAAUCGCGUGGAUAUGGCACGCAAGUUUCUACCGGGCAC
..........(..(.((((.((((..(((((.......)))))..........((((((.
CGUAAAUGUCCGACUAUGGGUGAGCAAUGGAACCGCACGUGUACGGUUUUUUGUGAUAUC
......)))))).....((((((((((((((((((........))))))...........
AGCAUUGCUUGCUCUUUAUUUGAGCGGGCAAUGCUUUUUUUA
..)))))))))))).)))).)))).)..).............`;

  function loadExample() {
    rnaInput = exampleRNA;
  }
</script>

<div class="rna-visualizer p-4 border rounded-lg shadow-sm">
  <h2 class="text-xl font-semibold mb-4">RNA Structure Visualization</h2>
  
  <div class="mb-4">
    <label for="rna-input" class="block mb-2 font-medium">RNA Sequence with Structure</label>
    <textarea 
      id="rna-input" 
      bind:value={rnaInput} 
      rows="8" 
      class="w-full p-2 border rounded-md font-mono text-sm"
      placeholder="Enter RNA sequence with structure annotation"
    ></textarea>
  </div>
  
  <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mb-6">
    <div>
      <label for="height-slider" class="block mb-1 font-medium">Height: {height}px</label>
      <input 
        id="height-slider" 
        type="range" 
        min="100" 
        max="900" 
        step="10" 
        bind:value={height} 
        class="w-full"
      />
    </div>
    
    <div>
      <label for="angle-slider" class="block mb-1 font-medium">Rotation Angle: {angle}°</label>
      <input 
        id="angle-slider" 
        type="range" 
        min="0" 
        max="360" 
        step="1" 
        bind:value={angle} 
        class="w-full"
      />
    </div>
    
    <div>
      <label for="color-theme" class="block mb-1 font-medium">Color Theme</label>
      <select 
        id="color-theme" 
        bind:value={colorTheme} 
        class="w-full p-2 border rounded-md"
      >
        <option value="default">Default</option>
        <option value="dark">Dark</option>
        <option value="white">White</option>
        <option value="black">Black</option>
        <option value="bright">Bright</option>
      </select>
    </div>
    
    <div class="flex items-end gap-4">
      <label class="flex items-center gap-2">
        <input type="checkbox" bind:checked={mirrorX} />
        <span>Flip X</span>
      </label>
      
      <label class="flex items-center gap-2">
        <input type="checkbox" bind:checked={mirrorY} />
        <span>Flip Y</span>
      </label>
    </div>
  </div>
  
  <div class="flex gap-4 mb-6">
    <button 
      on:click={visualizeRNA} 
      class="px-4 py-2 bg-blue-500 text-white rounded-md hover:bg-blue-600 focus:outline-none focus:ring-2 focus:ring-blue-300 disabled:opacity-50"
      disabled={loading || !rnaInput.trim()}
    >
      {loading ? 'Visualizing...' : 'Visualize RNA Structure'}
    </button>
    
    <button 
      on:click={loadExample} 
      class="px-4 py-2 bg-gray-200 text-gray-800 rounded-md hover:bg-gray-300 focus:outline-none focus:ring-2 focus:ring-gray-300"
    >
      Load Example
    </button>
  </div>
  
  {#if error}
    <div class="p-3 bg-red-100 text-red-800 rounded-md mb-4">
      {error}
    </div>
  {/if}
  
  {#if svgOutput}
    <div class="mt-6">
      <h3 class="text-lg font-medium mb-2">RNA Structure</h3>
      <div class="svg-container bg-white p-2 border rounded-md overflow-auto" bind:this={svgContainer}></div>
    </div>
  {/if}
</div>

<style>
  /* Monospace fonts look better for sequences */
  textarea {
    font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  }
  
  /* Make sure SVG is responsive */
  .svg-container {
    width: 100%;
    min-height: 300px;
    display: flex;
    justify-content: center;
  }
  
  .svg-container :global(svg) {
    max-width: 100%;
    height: auto;
  }
</style>