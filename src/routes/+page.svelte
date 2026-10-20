<div class="flex flex-1 flex-col gap-4 p-4 pt-0">
    <div class="h-24 rounded-xl p-4 shadow-sm">
        <h2 class="text-xl font-semibold mb-2">Welcome to Fennec</h2>
        <p class="text-muted-foreground">
            Fennec aims to let you run useful protein language models on your
            local machine.
        </p>
    </div>
    <div class="grid auto-rows-min gap-4 md:grid-cols-2">
        <div
            class="border-2 aspect-video rounded-xl p-4 shadow-sm"
        >
            <h3 class="text-lg font-semibold mb-2">Motivations</h3>
            <p class="text-muted-foreground">Protein Languge models are awesome</p>
            <p class="text-muted-foreground">Useful models are getting smaller</p>
            <p class="text-muted-foreground">
                Local Machines are getting more powerful
            </p>
            <p class="text-muted-foreground">
                Lets compile and run the locally/interactively!
            </p>
        </div>
        <div
            class="border-2 aspect-video rounded-xl p-4 shadow-sm"
        >
            <h3 class="text-lg font-semibold mb-2">Current State</h3>
            <p class="text-muted-foreground">
                - Design workspace: hover any residue in Mol* to see its ProteinMPNN / LigandMPNN substitution density, and colour the structure by entropy or by any amino acid's probability
                
            </p>
            <p class="text-muted-foreground">
                - Get the full the full set of sequence prodicions for an input
                protein using ESM2 or AMPLIFY
            </p>
            <p class="text-muted-foreground">
                - Get the predicted contact map from the attention heads using
                Amplify
            </p>
        </div>
    </div>
    <div class="grid auto-rows-min gap-4 md:grid-cols-2">
        <div
            class="border-2 aspect-video rounded-xl p-4 shadow-sm"
        >
            <h3 class="text-lg font-semibold mb-2">Links</h3>
            <p class="text-muted-foreground">
                Ferritin <a
                    href="https://github.com/ferritin-bio/ferritin"
                    target="_blank"
                    rel="noopener noreferrer">The Ferritin Github Repo</a
                >
            </p>
            <p class="text-muted-foreground">
                Fennec <a
                    href="https://github.com/ferritin-bio/fennec"
                    target="_blank"
                    rel="noopener noreferrer">The Fennec Github Repo</a
                >
            </p>
        </div>
        <div
            class="border-2 aspect-video rounded-xl p-4 shadow-sm"
        >
            <h3 class="text-lg font-semibold mb-2">Contribute</h3>
            <p class="text-muted-foreground">
                <a
                    href="https://github.com/ferritin-bio/ferritin/issues"
                    target="_blank"
                    rel="noopener noreferrer"
                    >Ferritin:
                </a> File an issue or start a discussion here
            </p>
            <p class="text-muted-foreground">
                <a
                    href="https://github.com/ferritin-bio/ferritin/issues"
                    target="_blank"
                    rel="noopener noreferrer">Fennec:</a
                > Feedback on this application
            </p>
            <p class="text-muted-foreground">
                <a
                    href="https://ferritin-bio.github.io/protein-language-models/"
                    target="_blank"
                    rel="noopener noreferrer"
                    >Protein-Language-Models:
                </a> Review / contribute to protien language models here.
            </p>
        </div>
    </div>
</div>

<style>
    a {
        color: hsl(var(--primary));
        text-decoration: underline;
    }
</style>
