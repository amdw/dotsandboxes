import { defineConfig } from "vitest/config";
import preact from "@preact/preset-vite";

export default defineConfig({
  plugins: [preact()],
  // Relative asset paths, so the built site can be served from any directory
  base: "./",
  // The analysis worker imports the WebAssembly module, which needs an ES module worker
  worker: {
    format: "es",
  },
  test: {
    environment: "node",
  },
});
