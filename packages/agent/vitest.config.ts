import { resolve } from "node:path"
import { defineConfig } from "vitest/config"

export default defineConfig({
  resolve: {
    alias: {
      "@better-npm/search": resolve(__dirname, "../search/src"),
      "@better-npm/agent": resolve(__dirname, "../agent/src"),
    },
  },
  test: {
    include: ["src/**/*.test.ts"],
    globals: true,
    environment: "node",
  },
})
