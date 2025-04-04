<!-- Sidebar and Navigation Controls here -->
<script lang="ts">
    import { page } from "$app/stores";
    import Microscope from "lucide-svelte/icons/microscope";
    import Molecule from "lucide-svelte/icons/component"; // Using component icon as a stand-in for molecule
    import SettingsIcon from "lucide-svelte/icons/settings"; // Changed to SettingsIcon
    import HelpIcon from "lucide-svelte/icons/help-circle"; // Changed to HelpIcon
    import FileText from "lucide-svelte/icons/file-text"; // For FASTA files
    import Dna from "lucide-svelte/icons/dna"; // For bioinformatics tools
    import Button from "./ui/button/button.svelte";

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
    <Sidebar.Content>
        <Sidebar.Header>
            <Button variant="outline" style="min-height: 50px" href="/">
                <img src="/app-icon.png" alt="Fennec" class="mr-2 size-6" />
                <span style="font-size: 16px">Fennec. Local-First ML </span>
            </Button>
        </Sidebar.Header>
        <Sidebar.Group>
            <Sidebar.GroupLabel>Deep Learning</Sidebar.GroupLabel>
            <Sidebar.GroupContent>
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
            </Sidebar.GroupContent>
        </Sidebar.Group>

        <Sidebar.Group>
            <Sidebar.GroupLabel>Bioinformatics Tools</Sidebar.GroupLabel>
            <Sidebar.GroupContent>
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
            </Sidebar.GroupContent>
        </Sidebar.Group>

        <!-- <Sidebar.Group>
            <Sidebar.GroupLabel>Utilities</Sidebar.GroupLabel>
            <Sidebar.GroupContent>
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
            </Sidebar.GroupContent>
        </Sidebar.Group>
         -->
    </Sidebar.Content>
</Sidebar.Root>

<style>
    .active {
        background-color: rgba(255, 255, 255, 0.1);
    }
    a {
        display: flex; /* Use flexbox */
        align-items: center; /* Vertically center items */
        gap: 0.75rem; /* Space between icon and text */
        padding: 0.5rem 1rem;
        text-decoration: none;
        color: inherit;
        width: 100%; /* Ensure the anchor takes full width */
    }

    a:hover {
        background-color: rgba(255, 255, 255, 0.05);
    }

    :global(svg) {
        width: 1.2em;
        height: 1.2em;
        flex-shrink: 0; /* Prevent icon from shrinking */
    }
    .card {
        background-color: #e5e5e5;
        border-radius: 8px;
        padding: 1rem;
        margin: 0.5rem;
    }

    .app-icon {
        width: 32px;
        height: 32px;
    }
</style>
