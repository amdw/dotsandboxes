// Load the real WebAssembly engine for unit tests running under Node
import { readFileSync } from "node:fs";
import { initEngineSync } from "../engine/engine";

let loaded = false;

export function loadTestEngine(): void {
  if (!loaded) {
    initEngineSync(readFileSync(new URL("../wasm/dabengine_wasm_bg.wasm", import.meta.url)));
    loaded = true;
  }
}
