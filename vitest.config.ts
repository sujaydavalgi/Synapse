import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    include: ["tests/**/*.test.ts"],
    globalSetup: ["scripts/vitest-global-setup.ts"],
    setupFiles: ["tests/vitest-setup.ts"],
  },
});
