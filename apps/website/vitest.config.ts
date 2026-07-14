import { defineConfig } from "vitest/config"

export default defineConfig({
  test: {
    environment: "jsdom",
    include: ["src/**/*.test.ts", "src/**/*.test.tsx", "app/**/*.test.ts", "app/**/*.test.tsx"],
    globals: true,
    setupFiles: ["../../packages/testing/src/setup.ts"],
    passWithNoTests: true,
  },
})
