import { resolve } from "node:path"
import { defineConfig } from "vitest/config"

export default defineConfig({
  resolve: {
    alias: {
      "@dash/search": resolve(__dirname, "../search/src"),
      "@dash/agent": resolve(__dirname, "../agent/src"),
    },
  },
  test: {
    include: ["src/**/*.test.ts"],
    globals: true,
    environment: "node",
  },
})
