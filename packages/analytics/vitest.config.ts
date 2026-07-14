import { resolve } from "node:path"
import { defineConfig } from "vitest/config"

export default defineConfig({
  resolve: {
    alias: {
      "@dash/sdk": resolve(__dirname, "../../packages/sdk/src"),
      "@dash/client": resolve(__dirname, "../../packages/client/src"),
      "@dash/types": resolve(__dirname, "../../packages/types/src"),
      "@dash/schemas": resolve(__dirname, "../../packages/schemas/src"),
      "@dash/design-system": resolve(__dirname, "../../packages/design-system/src"),
      "@dash/hooks": resolve(__dirname, "../../packages/hooks/src"),
      "@dash/utils": resolve(__dirname, "../../packages/utils/src"),
      "@dash/logger": resolve(__dirname, "../../packages/logger/src"),
      "@dash/config": resolve(__dirname, "../../packages/config/src"),
      "@dash/auth": resolve(__dirname, "../../packages/auth/src"),
      "@dash/validation": resolve(__dirname, "../../packages/validation/src"),
      "@dash/analytics": resolve(__dirname, "../../packages/analytics/src"),
      "@dash/telemetry": resolve(__dirname, "../../packages/telemetry/src"),
      "@dash/testing": resolve(__dirname, "../../packages/testing/src"),
      "@dash/shared": resolve(__dirname, "../../packages/shared/src"),
      "@dash/flags": resolve(__dirname, "../../packages/flags/src"),
      "@dash/storage": resolve(__dirname, "../../packages/storage/src"),
      "@dash/cache": resolve(__dirname, "../../packages/cache/src"),
      "@dash/contracts": resolve(__dirname, "../../packages/contracts/src"),
      "@dash/api": resolve(__dirname, "../../packages/api/src"),
      "@dash/mdx": resolve(__dirname, "../../packages/mdx/src"),
      "@dash/markdown": resolve(__dirname, "../../packages/markdown/src"),
      "@dash/search": resolve(__dirname, "../../packages/search/src"),
      "@dash/icons": resolve(__dirname, "../../packages/icons/src"),
    },
  },
  test: {
    include: ["src/**/*.test.ts", "src/**/*.test.tsx"],
    globals: true,
    environment: "node",
    setupFiles: ["../../packages/testing/src/setup.ts"],
  },
})
