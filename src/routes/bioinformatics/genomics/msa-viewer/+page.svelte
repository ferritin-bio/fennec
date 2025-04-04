<script>
  import MSAViewer from "$lib/components/genomics/MSAViewer.svelte";
</script>

<div class="container mx-auto p-4">
  <h1 class="text-2xl font-bold mb-6">Multiple Sequence Alignment Viewer</h1>
  
  <p class="mb-4 text-slate-600">
    The MSA Viewer allows visualization and analysis of multiple sequence alignments.
    It helps identify conserved regions, variations, and evolutionary relationships 
    between biological sequences.
  </p>
  
  <MSAViewer />
</div>