import { defineConfig } from "vitest/config";

export default defineConfig({
  // tsconfig.json extends SvelteKit's virtual $app/tsconfig, which only SvelteKit's
  // tooling resolves, so the test transform uses this standalone config instead
  tsconfig: "./tsconfig.vitest.json",
  test: {
    include: ["src/**/*.test.ts"],
    environment: "node",
  },
});
