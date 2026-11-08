// Web Worker running engine analyses, which can take a long time, off the main thread.
// The engine keeps its caches between analyses, so later ones can reuse earlier results.
//
// A full analysis is done in chunks (each a limited number of new positions), so that between
// chunks the worker can report progress, handle other requests (such as quick, limited
// analyses) and notice if the analysis has been cancelled. Each chunk caches everything it
// finishes, so the next carries on from there, and so does asking again after cancelling.
import init, * as wasm from "../wasm/dabengine_wasm.js";
import type { AnalysisKind } from "../game/analysis";
import type { PositionData } from "./types";
import type { AnalysisRequest, AnalysisResponse, WorkerMessage } from "./analyser";

// New positions per chunk of a full analysis: well under a second's work
const CHUNK_POSITIONS = 500_000;

const ready = init();
const cancelled = new Set<number>();

function analyse(kind: AnalysisKind, position: PositionData, maxNew: number | undefined, continuing: boolean) {
  return kind === "evaluate" ? wasm.evaluate(position, maxNew, continuing) : wasm.nimstringAnalysis(position, maxNew, continuing);
}

function reply(response: AnalysisResponse) {
  self.postMessage(response);
}

async function handle({ id, kind, position, maxNew }: AnalysisRequest) {
  try {
    await ready;
    if (maxNew !== undefined) {
      // A limited analysis, done in one go
      reply({ type: "result", id, ok: true, result: analyse(kind, position, maxNew, false) ?? null });
      return;
    }
    for (let chunk = 0; ; chunk++) {
      if (cancelled.delete(id)) {
        reply({ type: "result", id, ok: false, error: "cancelled" });
        return;
      }
      const result = analyse(kind, position, CHUNK_POSITIONS, chunk > 0);
      if (result) {
        reply({ type: "result", id, ok: true, result });
        return;
      }
      reply({ type: "progress", id, positions: wasm.cachedPositions(kind) });
      // Let other messages (cancellation, other requests) be handled before the next chunk
      await new Promise((resolve) => setTimeout(resolve, 0));
    }
  } catch (e) {
    reply({ type: "result", id, ok: false, error: e instanceof Error ? e.message : String(e) });
  }
}

self.onmessage = (event: MessageEvent<WorkerMessage>) => {
  const message = event.data;
  if (message.type === "cancel") cancelled.add(message.id);
  else void handle(message);
};
