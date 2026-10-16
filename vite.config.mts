import { resolve } from "node:path";
import { defineConfig } from "vite";

const ui = resolve(import.meta.dirname, "ui");

// The UI is two tiny local pages (the start field, and later settings). Tauri serves the built
// files from dist/; `tauri dev` uses the dev server on a fixed port.
export default defineConfig({
  root: ui,
  clearScreen: false,
  build: {
    outDir: resolve(import.meta.dirname, "dist"),
    emptyOutDir: true,
    target: "es2022",
    rollupOptions: {
      input: {
        index: resolve(ui, "index.html"),
      },
    },
  },
  server: {
    port: 1420,
    strictPort: true,
  },
});
