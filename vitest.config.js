import { defineConfig } from "vitest/config";

export default defineConfig({
  // tsconfig.json extends SvelteKit's virtual $app/tsconfig, which only SvelteKit's
  // generated config resolves, so point the transform at that file instead
  tsconfig: ".svelte-kit/tsconfig.json",
  test: {
    include: ["src/**/*.test.ts"],
    environment: "node",
  },
});
