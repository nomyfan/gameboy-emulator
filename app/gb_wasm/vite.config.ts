import { defineConfig } from "vite";
import wasm from "vite-plugin-wasm";
import react from "@vitejs/plugin-react";
import path from "node:path";

// https://vitejs.dev/config/
export default defineConfig({
  root: "web",
  build: {
    target: "esnext",
    outDir: path.resolve(__dirname, "dist"),
  },
  resolve: {
    alias: [
      {
        find: /^gb_wasm_bindings/,
        replacement: path.resolve(__dirname, "gb_wasm_bindings"),
      },
    ],
  },
  plugins: [wasm(), react()],
});
