import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { resolve } from "path";

export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  server: { port: 1420, strictPort: true },
  build: {
    target: "chrome110",
    rollupOptions: {
      input: {
        main: resolve(__dirname, "index.html"),
        quick: resolve(__dirname, "quick.html"),
        reminder: resolve(__dirname, "reminder.html"),
      },
    },
  },
});
