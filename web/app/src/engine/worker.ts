// Web Worker running engine analyses, which can take a long time, off the main thread.
// The engine keeps its caches between analyses, so later ones can reuse earlier results.
//
// A full analysis is done in chunks (each a limited number of new positions), so that between
// chunks the worker can report progress, handle other requests (such as quick, limited
// analyses) and notice if the analysis has been cancelled. Each chunk caches everything it
// finishes, so the next carries on from there, and so does asking again after cancelling.
//
// The engine's caches are limited in size, forgetting values when full (see MAX_CACHED_POSITIONS
// in api.rs), so in a very long analysis a chunk may forget what earlier ones found as fast as
// it finds anything new. So chunks grow in proportion to the work done so far, which ensures
// one is eventually long enough to finish.
import init, * as wasm from "../wasm/dabengine_wasm.js";
import type { AnalysisKind } from "../game/analysis";
import type { PositionData } from "./types";
import type { AnalysisRequest, AnalysisResponse, WorkerMessage } from "./analyser";

// New positions per chunk of a full analysis: at least this (well under a second's work), and
// at least 1/CHUNK_GROWTH of the positions analysed so far (so cancelling takes effect
// after at most that fraction of the time taken so far)
const CHUNK_POSITIONS = 500_000;
const CHUNK_GROWTH = 16;
// The most new positions that can be asked for at once (the engine takes a 32-bit count)
const MAX_CHUNK_POSITIONS = 2 ** 32 - 1;

const ready = init();
const cancelled = new Set<number>();

function analyse(kind: AnalysisKind, position: PositionData, maxNew: number) {
  return kind === "evaluate" ? wasm.evaluate(position, maxNew) : wasm.nimstringAnalysis(position, maxNew);
}

function reply(response: AnalysisResponse) {
  self.postMessage(response);
}

async function handle({ id, kind, position, maxNew }: AnalysisRequest) {
  try {
    await ready;
    if (maxNew !== undefined) {
      // A limited analysis, done in one go
      reply({ type: "result", id, ok: true, result: analyse(kind, position, maxNew) ?? null });
      return;
    }
    const start = wasm.positionsCalculated(kind);
    for (;;) {
      if (cancelled.delete(id)) {
        reply({ type: "result", id, ok: false, error: "cancelled" });
        return;
      }
      const done = wasm.positionsCalculated(kind) - start;
      const chunk = Math.min(Math.max(CHUNK_POSITIONS, Math.floor(done / CHUNK_GROWTH)), MAX_CHUNK_POSITIONS);
      const result = analyse(kind, position, chunk);
      if (result) {
        reply({ type: "result", id, ok: true, result });
        return;
      }
      reply({ type: "progress", id, positions: wasm.positionsCalculated(kind) - start });
      // Let other messages (cancellation, other requests) be handled before the next chunk
      await new Promise((resolve) => setTimeout(resolve, 0));
    }
  } catch (e) {
    if (e instanceof WebAssembly.RuntimeError) {
      // The engine crashed (e.g. "unreachable", from a panic or running out of memory),
      // which leaves it unusable, so this worker must be replaced (see Analyser)
      reply({ type: "result", id, ok: false, error: `The engine stopped unexpectedly, perhaps for lack of memory (${e.message})`, fatal: true });
    } else {
      reply({ type: "result", id, ok: false, error: e instanceof Error ? e.message : String(e) });
    }
  }
}

self.onmessage = (event: MessageEvent<WorkerMessage>) => {
  const message = event.data;
  if (message.type === "cancel") cancelled.add(message.id);
  else void handle(message);
};
