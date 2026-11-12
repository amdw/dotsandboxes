// Runs engine analyses in a Web Worker (see worker.ts), which keeps the engine's caches
// between them. Long analyses report their progress, and cancelling one stops it at the end
// of its current chunk, keeping everything it has calculated so far. If the engine crashes,
// every analysis waiting for it fails, and a new worker is started for the next.
import type { AnalysisKind } from "../game/analysis";
import type { Evaluation, NimstringAnalysis, PositionData } from "./types";

export interface AnalysisRequest {
  type: "analyse";
  id: number;
  kind: AnalysisKind;
  position: PositionData;
  // If given, give up (with a null result) if the analysis would need more than this many
  // positions which aren't already cached: i.e. only analyse if that's cheap
  maxNew?: number;
}

export type WorkerMessage = AnalysisRequest | { type: "cancel"; id: number };

type Result = Evaluation | NimstringAnalysis | null;

export type AnalysisResponse =
  | { type: "result"; id: number; ok: true; result: Result }
  // fatal if the engine crashed, so the worker can't be used again
  | { type: "result"; id: number; ok: false; error: string; fatal?: boolean }
  // A full analysis's progress: the number of positions it has analysed so far
  | { type: "progress"; id: number; positions: number };

export class CancelledError extends Error {
  constructor() {
    super("Analysis cancelled");
  }
}

interface Pending {
  full: boolean;
  resolve: (r: Result) => void;
  reject: (e: Error) => void;
  onProgress?: (positions: number) => void;
}

export class Analyser {
  private worker: Worker | null = null;
  private nextId = 0;
  private pending = new Map<number, Pending>();

  private getWorker(): Worker {
    if (!this.worker) {
      this.worker = new Worker(new URL("./worker.ts", import.meta.url), { type: "module" });
      this.worker.onmessage = (event: MessageEvent<AnalysisResponse>) => {
        const response = event.data;
        // (Even if the analysis was cancelled, the worker can't be used again)
        if (response.type === "result" && !response.ok && response.fatal) {
          this.fail(response.error);
          return;
        }
        const pending = this.pending.get(response.id);
        if (!pending) return;
        if (response.type === "progress") {
          pending.onProgress?.(response.positions);
          return;
        }
        this.pending.delete(response.id);
        if (response.ok) pending.resolve(response.result);
        else pending.reject(response.error === "cancelled" ? new CancelledError() : new Error(response.error));
      };
      this.worker.onerror = (event) => this.fail(event.message || "The engine stopped unexpectedly");
    }
    return this.worker;
  }

  // The worker is unusable (e.g. out of memory), so fail everything waiting for it, and
  // start a fresh one next time
  private fail(message: string): void {
    this.worker?.terminate();
    this.worker = null;
    const pending = [...this.pending.values()];
    this.pending.clear();
    for (const p of pending) p.reject(new Error(message));
  }

  run(kind: AnalysisKind, position: PositionData, options: { maxNew?: number; onProgress?: (positions: number) => void } = {}): Promise<Result> {
    const id = this.nextId++;
    return new Promise((resolve, reject) => {
      this.pending.set(id, { full: options.maxNew === undefined, resolve, reject, onProgress: options.onProgress });
      const request: AnalysisRequest = { type: "analyse", id, kind, position, maxNew: options.maxNew };
      this.getWorker().postMessage(request);
    });
  }

  // Stop all full analyses in progress or waiting (limited ones finish quickly anyway). They
  // stop at the end of their current chunk (the engine can't be interrupted part-way through
  // one), keeping what they've calculated; as far as the caller is concerned, they're
  // cancelled at once.
  cancel(): void {
    for (const [id, p] of this.pending) {
      if (!p.full) continue;
      this.worker?.postMessage({ type: "cancel", id } satisfies WorkerMessage);
      this.pending.delete(id);
      p.reject(new CancelledError());
    }
  }
}
