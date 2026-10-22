import { resolve } from "node:path"
import { defineConfig } from "vitest/config"

export default defineConfig({
  resolve: {
    alias: {
      "@better-npm/sdk": resolve(__dirname, "../../packages/sdk/src"),
      "@better-npm/client": resolve(__dirname, "../../packages/client/src"),
      "@better-npm/types": resolve(__dirname, "../../packages/types/src"),
      "@better-npm/schemas": resolve(__dirname, "../../packages/schemas/src"),
      "@better-npm/design-system": resolve(__dirname, "../../packages/design-system/src"),
      "@better-npm/hooks": resolve(__dirname, "../../packages/hooks/src"),
      "@better-npm/utils": resolve(__dirname, "../../packages/utils/src"),
      "@better-npm/logger": resolve(__dirname, "../../packages/logger/src"),
      "@better-npm/config": resolve(__dirname, "../../packages/config/src"),
      "@better-npm/auth": resolve(__dirname, "../../packages/auth/src"),
      "@better-npm/validation": resolve(__dirname, "../../packages/validation/src"),
      "@better-npm/analytics": resolve(__dirname, "../../packages/analytics/src"),
      "@better-npm/telemetry": resolve(__dirname, "../../packages/telemetry/src"),
      "@better-npm/testing": resolve(__dirname, "../../packages/testing/src"),
      "@better-npm/shared": resolve(__dirname, "../../packages/shared/src"),
      "@better-npm/flags": resolve(__dirname, "../../packages/flags/src"),
      "@better-npm/storage": resolve(__dirname, "../../packages/storage/src"),
      "@better-npm/cache": resolve(__dirname, "../../packages/cache/src"),
      "@better-npm/contracts": resolve(__dirname, "../../packages/contracts/src"),
      "@better-npm/api": resolve(__dirname, "../../packages/api/src"),
      "@better-npm/mdx": resolve(__dirname, "../../packages/mdx/src"),
      "@better-npm/markdown": resolve(__dirname, "../../packages/markdown/src"),
      "@better-npm/search": resolve(__dirname, "../../packages/search/src"),
      "@better-npm/icons": resolve(__dirname, "../../packages/icons/src"),
    },
  },
  test: {
    include: ["src/**/*.test.ts", "src/**/*.test.tsx"],
    globals: true,
    environment: "node",
    setupFiles: ["../../packages/testing/src/setup.ts"],
  },
})
