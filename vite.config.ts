import { fileURLToPath } from "node:url";
import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import tailwindcss from "@tailwindcss/vite";

const host = process.env.TAURI_DEV_HOST;

// `VITE_DEMO=1 pnpm tauri dev`: show the browser preview's demo data in the real app window.
const demoAlias = process.env.VITE_DEMO
  ? [
      { find: /^@tauri-apps\/api\/core$/, replacement: fileURLToPath(new URL("./src/dev/demo-tauri/core.ts", import.meta.url)) },
      { find: /^@tauri-apps\/api\/event$/, replacement: fileURLToPath(new URL("./src/dev/demo-tauri/event.ts", import.meta.url)) },
    ]
  : [];

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [svelte(), tailwindcss()],
  resolve: { alias: demoAlias },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
});
