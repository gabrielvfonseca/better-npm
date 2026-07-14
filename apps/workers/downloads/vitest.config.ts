import { defineConfig } from "vitest/config"

export default defineConfig({
  test: {
    include: ["src/**/*.test.ts"],
    globals: true,
    environment: "node",
    setupFiles: ["../../../packages/testing/src/setup.ts"],
    passWithNoTests: true,
  },
})
