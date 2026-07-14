import { defineConfig } from "vitest/config"

export default defineConfig({
  test: {
    include: ["./**/*.test.ts"],
    exclude: ["node_modules", "dist"],
    globals: true,
    environment: "node",
    testTimeout: 10_000,
    hookTimeout: 10_000,
    reporters: ["default", "verbose"],
  },
})
