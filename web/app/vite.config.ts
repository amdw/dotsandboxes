import { defineConfig } from "vitest/config";
import preact from "@preact/preset-vite";

export default defineConfig({
  plugins: [preact()],
  // Relative asset paths, so the built site can be served from any directory
  base: "./",
  test: {
    environment: "node",
  },
});
