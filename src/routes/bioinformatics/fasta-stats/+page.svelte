<script>
  import FastaStats from "$lib/components/FastaStats.svelte";
</script>

<div class="container mx-auto p-4">
  <h1 class="text-2xl font-bold mb-6">FASTA File Analysis</h1>
  
  <p class="mb-4 text-slate-600">
    This tool allows you to analyze FASTA files and view statistics about their content.
    Select a file to get started.
  </p>
  
  <FastaStats />
</div>