// Web Worker running engine analyses, which can take a long time, off the main thread.
// Cancelling an analysis means terminating this worker (see analyser.ts).
import init, * as wasm from "../wasm/dabengine_wasm.js";
import type { AnalysisRequest, AnalysisResponse } from "./analyser";

const ready = init();

self.onmessage = async (event: MessageEvent<AnalysisRequest>) => {
  const { id, kind, position } = event.data;
  let response: AnalysisResponse;
  try {
    await ready;
    const result = kind === "evaluate" ? wasm.evaluate(position) : wasm.nimstringAnalysis(position);
    response = { id, ok: true, result };
  } catch (e) {
    response = { id, ok: false, error: e instanceof Error ? e.message : String(e) };
  }
  self.postMessage(response);
};
