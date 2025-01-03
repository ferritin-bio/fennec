<script lang="ts">
    import { Input } from "$lib/components/ui/input/index.js";
    import { Button } from "$lib/components/ui/button/index.js";
    import ESM2Logits from "$lib/components/structure_components/esm2_logits.svelte";
    import AmplifyLogits from "$lib/components/structure_components/amplify_logits.svelte";

    // Stateful Variables
    let pdbCode = $state("");
    let loading = $state(false);
    let error = $state("");
    let pdb_text = $state("");

    const handleSubmit = async (event: Event) => {
        event.preventDefault();
        if (!pdbCode) return;
        loading = true;
        error = "";

        try {
            const response = await fetch(
                `https://files.rcsb.org/download/${pdbCode.toUpperCase()}.cif`,
            );
            if (!response.ok)
                throw new Error(`Failed to fetch: ${response.statusText}`);

            pdb_text = await response.text();
        } catch (err) {
            error =
                "Failed to load structure. Please check the PDB code and try again.";
            console.error("Error:", err);
        } finally {
            loading = false;
        }
    };
</script>

<div class="flex flex-col h-screen">
    <header class="flex h-16 shrink-0 items-center gap-2 border-b px-4">
        <form
            class="flex w-full max-w-sm items-center space-x-2"
            onsubmit={handleSubmit}
        >
            <Input
                type="text"
                placeholder="PDBCode"
                bind:value={pdbCode}
                disabled={loading}
            />
            <Button type="submit" disabled={loading}>
                {loading ? "Loading..." : "Retrieve"}
            </Button>
        </form>

        <div
            class="flex flex-row items-center space-x-4 p-2 bg-gray-100 rounded-md"
        >
            {#if pdbCode}
                <div class="info-item">
                    PDB: <span class="value">{pdbCode}</span>
                </div>
            {/if}
        </div>
    </header>

    {#if error}
        <p class="text-red-500 mt-2">{error}</p>
    {/if}

    <div class="flex-1 overflow-auto p-4">
        <div class="space-y-6">
            {#if pdb_text}
                <section>
                    <h2 class="text-2xl font-bold mb-4">ESM2</h2>
                    <div class="h-[400px] w-full">
                        <ESM2Logits pdbText={pdb_text} />
                    </div>
                </section>
                <section>
                    <h2 class="text-2xl font-bold mb-4">Amplify</h2>
                    <div class="h-[400px] w-full">
                        <AmplifyLogits pdbText={pdb_text} />
                    </div>
                </section>
            {/if}
        </div>
    </div>
</div>

<style>
    .info-item {
        @apply flex items-center;
    }
    .value {
        @apply text-red-500 ml-1;
    }
</style>
