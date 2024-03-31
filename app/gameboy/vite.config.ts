import * as path from "node:path";

import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

// https://vitejs.dev/config/
// eslint-disable-next-line import/no-default-export
export default defineConfig({
  server: {
    fs: {
      allow: ["../.."],
    },
  },
  plugins: [react()],
});
