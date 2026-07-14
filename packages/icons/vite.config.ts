import { resolve } from "node:path"
import react from "@vitejs/plugin-react"
import { defineConfig } from "vite"
import dts from "vite-plugin-dts"

export default defineConfig({
  plugins: [
    react(),
    dts({
      tsconfigPath: resolve(__dirname, "tsconfig.json"),
      entryRoot: resolve(__dirname, "src"),
    }),
  ],
  build: {
    lib: {
      entry: resolve(__dirname, "src/index.ts"),
      formats: ["es"],
    },
    outDir: "dist",
    sourcemap: true,
    rollupOptions: {
      external: [/^@dash\//, /^node:.*/, /^react/, /^react-dom/],
    },
  },
})
