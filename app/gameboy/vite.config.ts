import * as path from "node:path";

import react from "@vitejs/plugin-react";
import UnoCSS from "unocss/vite";
import { defineConfig } from "vite";
import { VitePWA } from "vite-plugin-pwa";
import svgr from "vite-plugin-svgr";

const __dirname = import.meta.dirname;

// https://vitejs.dev/config/
// eslint-disable-next-line import/no-default-export
export default defineConfig(({ mode }) => ({
  server: {
    fs: {
      allow: ["../.."],
    },
  },
  resolve: {
    alias: {
      gameboy: path.resolve(__dirname, "src"),
    },
  },
  define: {
    "process.env.NODE_ENV": JSON.stringify(mode),
  },
  plugins: [
    UnoCSS(),
    svgr(),
    react(),
    VitePWA({
      includeAssets: ["favicon.ico", "apple-touch-icon-180x180.png"],
      workbox: {
        globPatterns: ["**/*.{html,js,css,wasm}"],
      },
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
}));
