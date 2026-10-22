import { resolve } from "node:path"
import { defineConfig } from "vitest/config"

export default defineConfig({
  resolve: {
    alias: {
      "@better-npm/sdk": resolve(__dirname, "../sdk/src"),
      "@better-npm/client": resolve(__dirname, "../client/src"),
      "@better-npm/types": resolve(__dirname, "../types/src"),
      "@better-npm/schemas": resolve(__dirname, "../schemas/src"),
      "@better-npm/design-system": resolve(__dirname, "../ui/src"),
      "@better-npm/hooks": resolve(__dirname, "../hooks/src"),
      "@better-npm/utils": resolve(__dirname, "../utils/src"),
      "@better-npm/logger": resolve(__dirname, "../logger/src"),
      "@better-npm/config": resolve(__dirname, "../config/src"),
      "@better-npm/auth": resolve(__dirname, "../auth/src"),
      "@better-npm/validation": resolve(__dirname, "../validation/src"),
      "@better-npm/analytics": resolve(__dirname, "../analytics/src"),
      "@better-npm/telemetry": resolve(__dirname, "../telemetry/src"),
      "@better-npm/testing": resolve(__dirname, "../testing/src"),
      "@better-npm/shared": resolve(__dirname, "../shared/src"),
      "@better-npm/flags": resolve(__dirname, "../flags/src"),
      "@better-npm/storage": resolve(__dirname, "../storage/src"),
      "@better-npm/cache": resolve(__dirname, "../cache/src"),
      "@better-npm/contracts": resolve(__dirname, "../contracts/src"),
      "@better-npm/api": resolve(__dirname, "../api/src"),
      "@better-npm/mdx": resolve(__dirname, "../mdx/src"),
      "@better-npm/markdown": resolve(__dirname, "../markdown/src"),
      "@better-npm/search": resolve(__dirname, "../search/src"),
      "@better-npm/icons": resolve(__dirname, "../icons/src"),
      "@better-npm/agent": resolve(__dirname, "../agent/src"),
    },
  },
  test: {
    include: ["src/**/*.test.ts", "src/**/*.test.tsx"],
    globals: true,
    environment: "node",
  },
})
