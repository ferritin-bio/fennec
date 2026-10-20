<!-- Sidebar and Navigation Controls here -->
<script lang="ts">
    import { page } from "$app/state";
    import Atom from "@lucide/svelte/icons/atom";
    import Grid3x3 from "@lucide/svelte/icons/grid-3x3";
    import ChartColumn from "@lucide/svelte/icons/chart-column";
    import Microscope from "@lucide/svelte/icons/microscope";
    import FileText from "@lucide/svelte/icons/file-text";
    import Dna from "@lucide/svelte/icons/dna";
    import ChevronRight from "@lucide/svelte/icons/chevron-right";
    import * as Sidebar from "./ui/sidebar/index.js";
    import * as Collapsible from "#lib/components/ui/collapsible/index.js";

    type NavItem = { title: string; url: string; icon: typeof Atom };

    const designItems: NavItem[] = [
        { title: "Design workspace", url: "/structure_logits", icon: Atom },
        { title: "Sequence logits", url: "/sequence_logits", icon: ChartColumn },
        { title: "Contact map", url: "/contact_map", icon: Grid3x3 },
    ];

    const bioinformaticsItems: NavItem[] = [
        { title: "FASTA statistics", url: "/bioinformatics/fasta-stats", icon: FileText },
        { title: "DNA translator", url: "/bioinformatics/dna-translator", icon: Dna },
        { title: "RNA structure", url: "/bioinformatics/rna-structure", icon: Microscope },
    ];

    const genomicsItems: NavItem[] = [
        { title: "Chromosome viewer", url: "/bioinformatics/genomics/ideogram", icon: Dna },
        { title: "IGV browser", url: "/bioinformatics/genomics/igv-browser", icon: Microscope },
        { title: "MSA viewer", url: "/bioinformatics/genomics/msa-viewer", icon: FileText },
        { title: "Genome graph", url: "/bioinformatics/genomics/genome-graph", icon: Dna },
        { title: "Gosling", url: "/bioinformatics/genomics/gosling", icon: Microscope },
    ];

    const groups = [
        { label: "Protein design", items: designItems, open: true },
        { label: "Bioinformatics", items: bioinformaticsItems, open: false },
        { label: "Genomics", items: genomicsItems, open: false },
    ];
</script>

<Sidebar.Root collapsible="icon">
    <Sidebar.Header>
        <Sidebar.Menu>
            <Sidebar.MenuItem>
                <Sidebar.MenuButton size="lg" isActive={page.url.pathname === "/"}>
                    {#snippet child({ props })}
                        <a href="/" {...props}>
                            <img src="/app-icon.png" alt="" class="size-7 rounded-md" />
                            <span class="flex flex-col leading-tight">
                                <span class="font-semibold">Fennec</span>
                                <span class="text-muted-foreground text-xs">Local-first protein ML</span>
                            </span>
                        </a>
                    {/snippet}
                </Sidebar.MenuButton>
            </Sidebar.MenuItem>
        </Sidebar.Menu>
    </Sidebar.Header>

    <Sidebar.Content>
        {#each groups as group (group.label)}
            <Collapsible.Root open={group.open} class="group/collapsible">
                <Sidebar.Group>
                    <Sidebar.GroupLabel>
                        {#snippet child({ props })}
                            <Collapsible.Trigger {...props}>
                                {group.label}
                                <ChevronRight
                                    class="ml-auto transition-transform group-data-[state=open]/collapsible:rotate-90"
                                />
                            </Collapsible.Trigger>
                        {/snippet}
                    </Sidebar.GroupLabel>
                    <Collapsible.Content>
                        <Sidebar.Menu>
                            {#each group.items as item (item.url)}
                                {#snippet tooltip()}{item.title}{/snippet}
                                <Sidebar.MenuItem>
                                    <Sidebar.MenuButton
                                        isActive={page.url.pathname === item.url}
                                        tooltipContent={tooltip}
                                    >
                                        {#snippet child({ props })}
                                            <a href={item.url} {...props}>
                                                <item.icon />
                                                <span>{item.title}</span>
                                            </a>
                                        {/snippet}
                                    </Sidebar.MenuButton>
                                </Sidebar.MenuItem>
                            {/each}
                        </Sidebar.Menu>
                    </Collapsible.Content>
                </Sidebar.Group>
            </Collapsible.Root>
        {/each}
    </Sidebar.Content>
    <Sidebar.Rail />
</Sidebar.Root>
