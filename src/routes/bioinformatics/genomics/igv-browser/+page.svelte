<script>
  import IGVBrowser from "$lib/components/genomics/IGVBrowser.svelte";
</script>

<div class="container mx-auto p-4">
  <h1 class="text-2xl font-bold mb-6">Integrative Genomics Viewer (IGV)</h1>
  
  <p class="mb-4 text-slate-600">
    IGV is a high-performance, interactive tool for the visual exploration of genomic data.
    It supports a wide variety of data types including alignments, variants, gene expression, 
    and more.
  </p>
  
  <IGVBrowser />
</div>