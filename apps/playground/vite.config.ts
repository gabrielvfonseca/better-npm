import { resolve } from "node:path"
import { defineConfig } from "vite"

export default defineConfig({
  build: {
    ssr: true,
    outDir: "dist",
    sourcemap: true,
    rollupOptions: {
      input: resolve(__dirname, "src/index.ts"),
      external: [/^@dash\//, /^node:.*/, /^hono/, /^@hono\//, /^zod/],
    },
  },
})
