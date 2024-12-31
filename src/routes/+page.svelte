<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import StructureApp from "$lib/components/structure_explore.svelte";
    let name = $state("");
    let greetMsg = $state("");
    async function greet(event: Event) {
        event.preventDefault();
        greetMsg = await invoke("greet", { name });
    }
</script>

<StructureApp/>
