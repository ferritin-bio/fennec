<script>
  import { invoke } from '@tauri-apps/api/core';
  
  let dnaInput = '';
  let proteinOutput = '';
  let loading = false;
  let error = null;

  async function translateDna() {
    if (!dnaInput.trim()) {
      error = "Please enter DNA sequence";
      return;
    }
    
    try {
      loading = true;
      error = null;
      
      // Call the Rust function to translate DNA to protein
      proteinOutput = await invoke("translate_dna", { dnainput: dnaInput });
      
    } catch (err) {
      error = err.toString();
      console.error("Error translating DNA:", err);
    } finally {
      loading = false;
    }
  }

  // Example DNA sequence
  const exampleDna = `>NC_000913.3:1728347-1728994 rnt [organism=Escherichia coli str. K-12 substr. MG1655] [GeneID=946159] [chromosome=]
ATGTCCGATAACGCTCAACTTACCGGTCTGTGCGACCGTTTTCGTGGTTTTTATCCTGTTGTGATCGATGTTGAAACAGCCGGATTTAACGCCAAAACCGATGCGCTGCTTGAGATTGCCGCCATCACCCTGAAAATGGATGAACAAGGCTGGCTGATGCCGGACACCACATTACATTTCCACGTCGAACCATTTGTCGGCGCAAATTTGCAACCAGAAGCCCTCGCCTTCAACGGCATTGACCCGAACGATCCCGATCGCGGCGCGGTCAGCGAATACGAGGCGCTGCACGAAATTTTTAAAGTTGTACGTAAAGGTATTAAAGCGAGCGGCTGTAACCGCGCCATTATGGTGGCGCACAATGCCAATTTTGATCACAGCTTTATGATGGCCGCCGCAGAACGCGCCTCACTGAAACGTAACCCGTTCCACCCTTTCGCCACTTTTGACACTGCTGCACTGGCCGGGCTGGCACTCGGACAAACCGTATTGTCAAAGGCTTGCCAGACCGCTGGCATGGACTTCGACAGCACCCAGGCGCACTCCGCGCTGTACGACACCGAACGCACTGCTGTGCTGTTTTGTGAAATCGTCAACCGCTGGAAACGTCTGGGAGGCTGGCCGCTATCTGCCGCCGAAGAGGTGTAA`;

  function loadExample() {
    dnaInput = exampleDna;
  }
</script>

<div class="dna-translator p-4 border rounded-lg shadow-sm">
  <h2 class="text-xl font-semibold mb-4">DNA to Protein Translation</h2>
  
  <div class="mb-4">
    <label for="dna-input" class="block mb-2 font-medium">DNA Sequence</label>
    <textarea 
      id="dna-input" 
      bind:value={dnaInput} 
      rows="8" 
      class="w-full p-2 border rounded-md font-mono text-sm"
      placeholder="Enter DNA sequence or FASTA format"
    ></textarea>
  </div>
  
  <div class="flex gap-4 mb-6">
    <button 
      on:click={translateDna} 
      class="px-4 py-2 bg-blue-500 text-white rounded-md hover:bg-blue-600 focus:outline-none focus:ring-2 focus:ring-blue-300 disabled:opacity-50"
      disabled={loading || !dnaInput.trim()}
    >
      {loading ? 'Translating...' : 'Translate to Protein'}
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
  
  {#if proteinOutput}
    <div class="mt-6">
      <h3 class="text-lg font-medium mb-2">Protein Sequence</h3>
      <div class="bg-gray-50 p-4 rounded-md">
        <pre class="whitespace-pre-wrap font-mono text-sm">{proteinOutput}</pre>
      </div>
    </div>
  {/if}
</div>

<style>
  /* Monospace fonts look better for sequences */
  textarea, pre {
    font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  }
</style>