<!-- Sidebar and Navigation Controls here -->
<script lang="ts">
    import { page } from "$app/stores";
    import Microscope from "lucide-svelte/icons/microscope";
    import Molecule from "lucide-svelte/icons/component"; // Using component icon as a stand-in for molecule
    import SettingsIcon from "lucide-svelte/icons/settings"; // Changed to SettingsIcon
    import HelpIcon from "lucide-svelte/icons/help-circle"; // Changed to HelpIcon
    import FileText from "lucide-svelte/icons/file-text"; // For FASTA files
    import Dna from "lucide-svelte/icons/dna"; // For bioinformatics tools
    import { ChevronDown } from "lucide-svelte";
    import Button from "./ui/button/button.svelte";
    import * as Sidebar from "./ui/sidebar/index.js";
    import * as Card from "$lib/components/ui/card/index.js";
    import * as Avatar from "$lib/components/ui/avatar/index.js";
    import * as Collapsible from "$lib/components/ui/collapsible/index.js";

    // Track which groups are open
    let isDeepLearningOpen = true;
    let isBioinformaticsOpen = false;
    let isUtilitiesOpen = false;

    // Menu items for viewers
    const viewerItems = [
        {
            title: "Explore Structures: LigandMPNN",
            url: "/structure_logits",
            icon: Molecule,
        },
        {
            title: "Sequence Logits",
            url: "/sequence_logits",
            icon: Molecule,
        },
        {
            title: "Contact Map",
            url: "/contact_map",
            icon: Molecule,
        },
    ];

    // New bioinformatics tools from Observable
    const bioInformaticsItems = [
        {
            title: "FASTA Statistics",
            url: "/bioinformatics/fasta-stats",
            icon: FileText,
        },
        {
            title: "DNA Translator",
            url: "/bioinformatics/dna-translator",
            icon: Dna,
        },
        {
            title: "RNA Structure",
            url: "/bioinformatics/rna-structure",
            icon: Microscope,
        },
        // Genomics visualization tools
        {
            title: "Chromosome Viewer",
            url: "/bioinformatics/genomics/ideogram",
            icon: Dna,
        },
        {
            title: "IGV Browser",
            url: "/bioinformatics/genomics/igv-browser",
            icon: Microscope,
        },
        {
            title: "MSA Viewer",
            url: "/bioinformatics/genomics/msa-viewer",
            icon: FileText,
        },
        {
            title: "Genome Graph",
            url: "/bioinformatics/genomics/genome-graph",
            icon: Dna,
        },
        {
            title: "Genome Ribbon",
            url: "/bioinformatics/genomics/genome-ribbon",
            icon: Dna,
        },
        {
            title: "Gosling Visualizer",
            url: "/bioinformatics/genomics/gosling",
            icon: Microscope,
        },
    ];

    const utilityItems = [
        {
            title: "Settings",
            url: "/settings",
            icon: SettingsIcon,
        },
        {
            title: "Help",
            url: "/help",
            icon: HelpIcon,
        },
    ];
</script>

<Sidebar.Root>
    <Sidebar.Content class="sidebar-content">
        <Sidebar.Header>
            <Button variant="outline" style="min-height: 50px" href="/">
                <img src="/app-icon.png" alt="Fennec" class="mr-2 size-6" />
                <span style="font-size: 16px">Fennec. Local-First ML </span>
            </Button>
        </Sidebar.Header>

        <!-- Deep Learning Section with Collapsible -->
        <div class="sidebar-section">
            <Collapsible.Root bind:open={isDeepLearningOpen}>
                <Collapsible.Trigger class="sidebar-group-header">
                    <div class="section-title-wrapper">
                        <span class="section-title">Deep Learning</span>
                        <ChevronDown
                            class="h-4 w-4 transition-transform duration-200"
                            style="transform: {isDeepLearningOpen
                                ? 'rotate(0deg)'
                                : 'rotate(-90deg)'}"
                        />
                    </div>
                </Collapsible.Trigger>
                <Collapsible.Content class="collapsible-section-content">
                    <Sidebar.Menu>
                        {#each viewerItems as item (item.title)}
                            <Sidebar.MenuItem>
                                <Sidebar.MenuButton>
                                    {#snippet child({ props })}
                                        <a
                                            href={item.url}
                                            {...props}
                                            class:active={$page.url.pathname ===
                                                item.url}
                                        >
                                            <item.icon />
                                            <span>{item.title}</span>
                                        </a>
                                    {/snippet}
                                </Sidebar.MenuButton>
                            </Sidebar.MenuItem>
                        {/each}
                    </Sidebar.Menu>
                </Collapsible.Content>
            </Collapsible.Root>
        </div>

        <!-- Bioinformatics Tools Section with Collapsible -->
        <div class="sidebar-section">
            <Collapsible.Root bind:open={isBioinformaticsOpen}>
                <Collapsible.Trigger class="sidebar-group-header">
                    <div class="section-title-wrapper">
                        <span class="section-title">Bioinformatics Tools</span>
                        <ChevronDown
                            class="h-4 w-4 transition-transform duration-200"
                            style="transform: {isBioinformaticsOpen
                                ? 'rotate(0deg)'
                                : 'rotate(-90deg)'}"
                        />
                    </div>
                </Collapsible.Trigger>
                <Collapsible.Content class="collapsible-section-content">
                    <Sidebar.Menu>
                        {#each bioInformaticsItems as item (item.title)}
                            <Sidebar.MenuItem>
                                <Sidebar.MenuButton>
                                    {#snippet child({ props })}
                                        <a
                                            href={item.url}
                                            {...props}
                                            class:active={$page.url.pathname ===
                                                item.url}
                                        >
                                            <item.icon />
                                            <span>{item.title}</span>
                                        </a>
                                    {/snippet}
                                </Sidebar.MenuButton>
                            </Sidebar.MenuItem>
                        {/each}
                    </Sidebar.Menu>
                </Collapsible.Content>
            </Collapsible.Root>
        </div>

        <!-- Utilities Section with Collapsible -->
        <div class="sidebar-section">
            <Collapsible.Root bind:open={isUtilitiesOpen}>
                <Collapsible.Trigger class="sidebar-group-header">
                    <div class="section-title-wrapper">
                        <span class="section-title">Utilities</span>
                        <ChevronDown
                            class="h-4 w-4 transition-transform duration-200"
                            style="transform: {isUtilitiesOpen
                                ? 'rotate(0deg)'
                                : 'rotate(-90deg)'}"
                        />
                    </div>
                </Collapsible.Trigger>
                <Collapsible.Content class="collapsible-section-content">
                    <Sidebar.Menu>
                        {#each utilityItems as item (item.title)}
                            <Sidebar.MenuItem>
                                <Sidebar.MenuButton>
                                    {#snippet child({ props })}
                                        <a
                                            href={item.url}
                                            {...props}
                                            class:active={$page.url.pathname ===
                                                item.url}
                                        >
                                            <item.icon />
                                            <span>{item.title}</span>
                                        </a>
                                    {/snippet}
                                </Sidebar.MenuButton>
                            </Sidebar.MenuItem>
                        {/each}
                    </Sidebar.Menu>
                </Collapsible.Content>
            </Collapsible.Root>
        </div>
    </Sidebar.Content>
</Sidebar.Root>

<style>
    /* Left-align all buttons, triggers, and other elements that might be generating the IDs */
    :global([id^="bits-"]) {
        text-align: left !important;
    }

    /* Add styles for the sidebar section container */
    .sidebar-section {
        width: 100%;
        text-align: left;
        margin-top: 0.5rem;
    }

    /* Add spacing to the sidebar content */
    :global(.sidebar-content) {
        padding: 0.75rem;
    }

    .active {
        background-color: rgba(255, 255, 255, 0.1);
    }

    a {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        padding: 0.5rem 1rem;
        text-decoration: none;
        color: inherit;
        width: 100%;
        border-radius: 0.25rem;
    }

    a:hover {
        background-color: rgba(255, 255, 255, 0.05);
    }

    :global(svg) {
        width: 1.2em;
        height: 1.2em;
        flex-shrink: 0;
    }

    .sidebar-group-header {
        width: 100%;
        padding: 0.75rem 1rem;
        text-align: left !important;
        background: transparent;
        border: none;
        cursor: pointer;
        border-radius: 0.25rem;
        margin-bottom: 0.25rem;
    }

    .sidebar-group-header:hover {
        background-color: rgba(255, 255, 255, 0.05);
    }

    /* Wrapper to handle layout */
    .section-title-wrapper {
        display: flex;
        justify-content: space-between;
        align-items: center;
        width: 100%;
        text-align: left !important;
    }

    /* Explicitly left-align the title */
    .section-title {
        text-align: left !important;
        font-weight: 500;
        font-size: 0.95rem;
    }

    /* Ensure all button content is left-aligned */
    :global(button) {
        text-align: left !important;
    }

    /* Add some spacing and visual hierarchy to the sidebar sections */
    .collapsible-section-content {
        padding-left: 0.75rem;
        padding-right: 0.5rem;
        margin-top: 0.25rem;
        margin-bottom: 0.5rem;
    }

    /* Add animation for the collapsible content */
    :global(.collapsible-content) {
        overflow: hidden;
        transition: height 300ms cubic-bezier(0.87, 0, 0.13, 1);
    }
</style>
