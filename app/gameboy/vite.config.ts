import { vanillaExtractPlugin } from "@vanilla-extract/vite-plugin";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";
import svgr from "vite-plugin-svgr";
import tsconfigPaths from "vite-tsconfig-paths";

// https://vitejs.dev/config/
// eslint-disable-next-line import/no-default-export
export default defineConfig({
  server: {
    fs: {
      allow: ["../.."],
    },
  },
  plugins: [tsconfigPaths(), vanillaExtractPlugin(), svgr(), react()],
  build: {
    rollupOptions: {
      output: {
        manualChunks: (id) => {
          const matchers = [
            {
              reg: /\/(react-dom)|(react)\//,
              name: "react",
            },
            {
              reg: /\/react-spinners\//,
              name: "spinners",
            },
            {
              reg: /\/(zustand)|(immer)\//,
              name: "store",
            },
            {
              reg: /\/(dexie)\//,
              name: "storage",
            },
            {
              reg: /\/@radix-ui\//,
              name: "ui",
            },
            {
              reg: /\/rxjs\//,
              name: "rx",
            },
            {
              reg: /\/swr\//,
              name: "swr",
            },
          ];
          for (const matcher of matchers) {
            if (matcher.reg.test(id)) {
              return matcher.name;
            }
          }
        },
      },
    },
  },
});
