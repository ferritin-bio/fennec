<script lang="ts">
    import Molstar from "$lib/components/molstar.svelte";
    import { invoke } from "@tauri-apps/api/core";
    import MiewApp from "$lib/components/miew.svelte";
    let name = $state("");
    let greetMsg = $state("");
    async function greet(event: Event) {
        event.preventDefault();
        greetMsg = await invoke("greet", { name });
    }
</script>

<MiewApp />
<!-- <Molstar /> -->
