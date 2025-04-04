<script>
  import GenomeGraph from "$lib/components/genomics/GenomeGraph.svelte";
</script>

<div class="container mx-auto p-4">
  <h1 class="text-2xl font-bold mb-6">Genome Graph Viewer</h1>
  
  <p class="mb-4 text-slate-600">
    The Genome Graph Viewer allows visualization of genomic data in graph format.
    This is useful for representing complex genomic structures that don't fit well 
    into linear representations.
  </p>
  
  <GenomeGraph />
</div>