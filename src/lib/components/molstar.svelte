<script lang="ts">
    import { onMount } from "svelte";
    import Calendar from "lucide-svelte/icons/calendar";
    import House from "lucide-svelte/icons/house";
    import Inbox from "lucide-svelte/icons/inbox";
    import Search from "lucide-svelte/icons/search";
    import Settings from "lucide-svelte/icons/settings";
    import * as Sidebar from "$lib/components/ui/sidebar/index.js";

    // @ts-ignore
    import * as molstar from "molstar/build/viewer/molstar.js";
    import * as molstarcss from "molstar/build/viewer/molstar.css";

    onMount(() => {
        molstar.Viewer.create("app", {
            layoutIsExpanded: false,
            layoutShowControls: false,
            layoutShowRemoteState: false,
            layoutShowSequence: true,
            layoutShowLog: false,
            layoutShowLeftPanel: true,
            viewportShowExpand: true,
            viewportShowSelectionMode: true,
            viewportShowAnimation: false,
            pdbProvider: "rcsb",
            emdbProvider: "rcsb",
        }).then(
            // @ts-ignore
            (viewer) => {
                viewer.loadPdb("7bv2");
                // molstar_viewer = viewer;
                window.molstar_viewer = viewer;
            },
        );
    });
</script>

<div id="app"></div>

<style>
    #app {
        position: absolute;
        left: 100px;
        top: 100px;
        width: 800px;
        height: 600px;
    }
</style>
