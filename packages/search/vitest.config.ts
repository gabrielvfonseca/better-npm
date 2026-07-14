import { resolve } from "node:path"
import { defineConfig } from "vitest/config"

export default defineConfig({
  resolve: {
    alias: {
      "@dash/sdk": resolve(__dirname, "../sdk/src"),
      "@dash/client": resolve(__dirname, "../client/src"),
      "@dash/types": resolve(__dirname, "../types/src"),
      "@dash/schemas": resolve(__dirname, "../schemas/src"),
      "@dash/design-system": resolve(__dirname, "../ui/src"),
      "@dash/hooks": resolve(__dirname, "../hooks/src"),
      "@dash/utils": resolve(__dirname, "../utils/src"),
      "@dash/logger": resolve(__dirname, "../logger/src"),
      "@dash/config": resolve(__dirname, "../config/src"),
      "@dash/auth": resolve(__dirname, "../auth/src"),
      "@dash/validation": resolve(__dirname, "../validation/src"),
      "@dash/analytics": resolve(__dirname, "../analytics/src"),
      "@dash/telemetry": resolve(__dirname, "../telemetry/src"),
      "@dash/testing": resolve(__dirname, "../testing/src"),
      "@dash/shared": resolve(__dirname, "../shared/src"),
      "@dash/flags": resolve(__dirname, "../flags/src"),
      "@dash/storage": resolve(__dirname, "../storage/src"),
      "@dash/cache": resolve(__dirname, "../cache/src"),
      "@dash/contracts": resolve(__dirname, "../contracts/src"),
      "@dash/api": resolve(__dirname, "../api/src"),
      "@dash/mdx": resolve(__dirname, "../mdx/src"),
      "@dash/markdown": resolve(__dirname, "../markdown/src"),
      "@dash/search": resolve(__dirname, "../search/src"),
      "@dash/icons": resolve(__dirname, "../icons/src"),
      "@dash/agent": resolve(__dirname, "../agent/src"),
    },
  },
  test: {
    include: ["src/**/*.test.ts", "src/**/*.test.tsx"],
    globals: true,
    environment: "node",
  },
})
