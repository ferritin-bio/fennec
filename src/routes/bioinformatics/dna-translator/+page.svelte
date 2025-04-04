<script>
  import DnaTranslator from "$lib/components/DnaTranslator.svelte";
</script>

<div class="container mx-auto p-4">
  <h1 class="text-2xl font-bold mb-6">DNA to Protein Translation</h1>
  
  <p class="mb-4 text-slate-600">
    This tool allows you to translate DNA sequences into protein sequences using the standard genetic code.
    Enter your DNA sequence or load the example to get started.
  </p>
  
  <DnaTranslator />
</div>