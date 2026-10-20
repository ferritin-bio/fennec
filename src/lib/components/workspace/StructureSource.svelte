<!-- Load a structure into the shared store from RCSB, AlphaFold DB, or a local file. -->
<script lang="ts">
    import Upload from "@lucide/svelte/icons/upload";
    import { Input } from "#lib/components/ui/input/index.js";
    import { Button } from "#lib/components/ui/button/index.js";
    import { structure } from "#lib/structure/store.svelte.js";

    let source = $state<"pdb" | "afdb">("pdb");
    let query = $state("");
    let fileInput: HTMLInputElement;

    const examples = { pdb: ["1BC8", "101M", "6M0J"], afdb: ["P69905", "P00698"] };

    function submit(e: Event) {
        e.preventDefault();
        if (!query.trim()) return;
        if (source === "pdb") structure.loadPdb(query);
        else structure.loadAlphaFold(query);
    }
</script>

<form class="flex flex-wrap items-center gap-2" onsubmit={submit}>
    <div class="bg-muted inline-flex rounded-md p-0.5 text-xs" role="radiogroup" aria-label="Structure source">
        {#each [["pdb", "PDB"], ["afdb", "AlphaFold"]] as [id, label] (id)}
            <button
                type="button"
                role="radio"
                aria-checked={source === id}
                class="rounded px-2 py-1 {source === id ? 'bg-background shadow-sm' : 'text-muted-foreground'}"
                onclick={() => (source = id as "pdb" | "afdb")}>{label}</button
            >
        {/each}
    </div>
    <Input
        class="h-8 w-36 font-mono uppercase"
        aria-label={source === "pdb" ? "PDB id" : "UniProt accession"}
        placeholder={source === "pdb" ? "PDB id" : "UniProt acc."}
        bind:value={query}
        disabled={structure.loading}
    />
    <Button type="submit" size="sm" disabled={structure.loading || !query.trim()}>
        {structure.loading ? "Loading…" : "Load"}
    </Button>
    <Button type="button" size="sm" variant="outline" onclick={() => fileInput.click()} disabled={structure.loading}>
        <Upload class="size-4" /> File
    </Button>
    <input
        bind:this={fileInput}
        type="file"
        accept=".cif,.mmcif,.pdb,.ent"
        class="hidden"
        onchange={(e) => {
            const f = e.currentTarget.files?.[0];
            if (f) structure.loadFile(f);
            e.currentTarget.value = "";
        }}
    />
    <span class="text-muted-foreground hidden text-xs lg:inline">
        e.g.
        {#each examples[source] as ex, i (ex)}
            <button
                type="button"
                class="hover:text-foreground font-mono underline-offset-2 hover:underline"
                onclick={() => {
                    query = ex;
                    if (source === "pdb") structure.loadPdb(ex);
                    else structure.loadAlphaFold(ex);
                }}>{ex}</button
            >{i < examples[source].length - 1 ? ", " : ""}
        {/each}
    </span>
</form>
{#if structure.error}
    <p class="text-destructive mt-1 text-xs" role="alert">{structure.error}</p>
{/if}
