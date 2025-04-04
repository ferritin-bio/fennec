<script>
  import { invoke } from '@tauri-apps/api/core';
  import { open } from '@tauri-apps/plugin-dialog';

  let stats = null;
  let loading = false;
  let error = null;

  async function getFastaStats() {
    try {
      loading = true;
      error = null;
      
      // Open file dialog for the user to select a FASTA file
      const fasta = await open({
        multiple: false,
        filters: [
          {
            name: "Fasta",
            extensions: ["fa", "fasta", "fna"],
          },
        ],
      });
      
      // User might cancel the file selection
      if (!fasta) {
        loading = false;
        return;
      }
      
      // Call the Rust function to analyze the FASTA file
      stats = await invoke("get_stats", { filename: fasta });
      
    } catch (err) {
      error = err.toString();
      console.error("Error getting FASTA stats:", err);
    } finally {
      loading = false;
    }
  }

  async function getDetailedFastaStats() {
    try {
      loading = true;
      error = null;
      
      const fasta = await open({
        multiple: false,
        filters: [
          {
            name: "Fasta",
            extensions: ["fa", "fasta", "fna"],
          },
        ],
      });
      
      if (!fasta) {
        loading = false;
        return;
      }
      
      // Call the more detailed stats function
      stats = await invoke("get_seqstats", { filename: fasta });
      
    } catch (err) {
      error = err.toString();
      console.error("Error getting detailed FASTA stats:", err);
    } finally {
      loading = false;
    }
  }
</script>

<div class="fasta-stats-container p-4 border rounded-lg shadow-sm">
  <h2 class="text-xl font-semibold mb-4">FASTA Statistics</h2>
  
  <div class="flex gap-4 mb-6">
    <button 
      on:click={getFastaStats} 
      class="px-4 py-2 bg-blue-500 text-white rounded-md hover:bg-blue-600 focus:outline-none focus:ring-2 focus:ring-blue-300 disabled:opacity-50"
      disabled={loading}
    >
      {loading ? 'Loading...' : 'Basic Stats'}
    </button>
    
    <button 
      on:click={getDetailedFastaStats} 
      class="px-4 py-2 bg-green-500 text-white rounded-md hover:bg-green-600 focus:outline-none focus:ring-2 focus:ring-green-300 disabled:opacity-50"
      disabled={loading}
    >
      {loading ? 'Loading...' : 'Detailed Stats'}
    </button>
  </div>
  
  {#if error}
    <div class="p-3 bg-red-100 text-red-800 rounded-md mb-4">
      {error}
    </div>
  {/if}
  
  {#if stats}
    <div class="stats-results p-4 bg-gray-50 rounded-md">
      <h3 class="text-lg font-medium mb-2">Results</h3>
      
      {#if 'recordcount' in stats}
        <!-- Basic stats -->
        <div class="grid grid-cols-2 gap-2">
          <div class="stat-label font-medium">Record Count:</div>
          <div>{stats.recordcount}</div>
          
          <div class="stat-label font-medium">Maximum Length:</div>
          <div>{stats.maxlength}</div>
        </div>
      {:else}
        <!-- Detailed stats -->
        <div class="grid grid-cols-2 gap-2">
          <div class="stat-label font-medium">File:</div>
          <div>{stats.filename}</div>
          
          <div class="stat-label font-medium">Format:</div>
          <div>{stats.format}</div>
          
          <div class="stat-label font-medium">Type:</div>
          <div>{stats.datatype}</div>
          
          <div class="stat-label font-medium">Sequences:</div>
          <div>{stats.num_seqs}</div>
          
          <div class="stat-label font-medium">Total Length:</div>
          <div>{stats.sum_len}</div>
          
          <div class="stat-label font-medium">Min Length:</div>
          <div>{stats.min_len}</div>
          
          <div class="stat-label font-medium">Average Length:</div>
          <div>{stats.avg_len.toFixed(2)}</div>
          
          <div class="stat-label font-medium">Max Length:</div>
          <div>{stats.max_len}</div>
        </div>
        
        <!-- We could add a histogram plot here later -->
      {/if}
    </div>
  {/if}
</div>

<style>
  /* Add any component-specific styles here if needed */
</style>