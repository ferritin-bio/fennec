<script lang="ts">
    import "../app.css";
    import type { Snippet } from "svelte";
    import * as Sidebar from "#lib/components/ui/sidebar/index.js";
    import AppSidebar from "#lib/components/app-sidebar.svelte";
    import { initTheme } from "#lib/theme.svelte.js";

    const { children }: { children: Snippet } = $props();

    initTheme();
</script>

<Sidebar.Provider>
    <AppSidebar />
    <Sidebar.Inset class="min-h-svh">
        <header class="bg-background/80 sticky top-0 z-10 flex h-11 shrink-0 items-center gap-2 border-b px-3 backdrop-blur">
            <Sidebar.Trigger />
        </header>
        <div class="min-h-0 flex-1">
            {@render children()}
        </div>
    </Sidebar.Inset>
</Sidebar.Provider>
