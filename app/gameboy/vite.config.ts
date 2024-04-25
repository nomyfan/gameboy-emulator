import { vanillaExtractPlugin } from "@vanilla-extract/vite-plugin";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";
import { VitePWA } from "vite-plugin-pwa";
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
  plugins: [
    tsconfigPaths(),
    vanillaExtractPlugin(),
    svgr(),
    react(),
    VitePWA({
      devOptions: {
        enabled: true,
        type: "module",
      },
      includeAssets: ["favicon.ico", "apple-touch-icon-180x180.png"],
      manifest: {
        name: "GameBoy",
        short_name: "GameBoy",
        description: "A GameBoy emulator",
        theme_color: "#EBEBEB",
        background_color: "#EBEBEB",
        orientation: "landscape",
        icons: [
          {
            src: "pwa-64x64.png",
            sizes: "64x64",
            type: "image/png",
          },
          {
            src: "pwa-192x192.png",
            sizes: "192x192",
            type: "image/png",
          },
          {
            src: "pwa-512x512.png",
            sizes: "512x512",
            type: "image/png",
          },
          {
            src: "maskable-icon-512x512.png",
            sizes: "512x512",
            type: "image/png",
            purpose: "maskable",
          },
        ],
      },
    }),
  ],
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
