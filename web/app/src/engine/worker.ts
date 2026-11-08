// Web Worker running engine analyses, which can take a long time, off the main thread.
// The engine keeps its caches between analyses, so later ones can reuse earlier results.
// Cancelling an analysis means terminating this worker (see analyser.ts).
import init, * as wasm from "../wasm/dabengine_wasm.js";
import type { AnalysisRequest, AnalysisResponse } from "./analyser";

const ready = init();

self.onmessage = async (event: MessageEvent<AnalysisRequest>) => {
  const { id, kind, position, maxNew } = event.data;
  let response: AnalysisResponse;
  try {
    await ready;
    const result = kind === "evaluate" ? wasm.evaluate(position, maxNew) : wasm.nimstringAnalysis(position, maxNew);
    response = { id, ok: true, result: result ?? null };
  } catch (e) {
    response = { id, ok: false, error: e instanceof Error ? e.message : String(e) };
  }
  self.postMessage(response);
};
