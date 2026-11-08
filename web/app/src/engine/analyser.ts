// Runs engine analyses in a Web Worker. Requests are handled in turn; the worker keeps the
// engine's caches between them, so cancelling (which terminates the worker, the only way to
// interrupt the engine) also discards the caches.
import type { AnalysisKind } from "../game/analysis";
import type { Evaluation, NimstringAnalysis, PositionData } from "./types";

export interface AnalysisRequest {
  id: number;
  kind: AnalysisKind;
  position: PositionData;
  // If given, give up (with a null result) if the analysis would need more than this many
  // positions which aren't already cached: i.e. only analyse if that's cheap
  maxNew?: number;
}

type Result = Evaluation | NimstringAnalysis | null;

export type AnalysisResponse = { id: number; ok: true; result: Result } | { id: number; ok: false; error: string };

export class CancelledError extends Error {
  constructor() {
    super("Analysis cancelled");
  }
}

interface Pending {
  resolve: (r: Result) => void;
  reject: (e: Error) => void;
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
        const pending = this.pending.get(response.id);
        if (!pending) return;
        this.pending.delete(response.id);
        if (response.ok) pending.resolve(response.result);
        else pending.reject(new Error(response.error));
      };
      this.worker.onerror = (event) => {
        // The worker may be unusable (e.g. out of memory), so start a fresh one next time
        this.stop(new Error(event.message || "The engine stopped unexpectedly"));
      };
    }
    return this.worker;
  }

  evaluate(position: PositionData, maxNew?: number): Promise<Evaluation | null> {
    return this.run("evaluate", position, maxNew) as Promise<Evaluation | null>;
  }

  nimstringAnalysis(position: PositionData, maxNew?: number): Promise<NimstringAnalysis | null> {
    return this.run("nimstring", position, maxNew) as Promise<NimstringAnalysis | null>;
  }

  run(kind: AnalysisKind, position: PositionData, maxNew?: number): Promise<Result> {
    const id = this.nextId++;
    return new Promise((resolve, reject) => {
      this.pending.set(id, { resolve, reject });
      const request: AnalysisRequest = { id, kind, position, maxNew };
      this.getWorker().postMessage(request);
    });
  }

  // Stop all analyses in progress or waiting. The engine can't be interrupted, so the worker
  // is terminated (losing its caches), and a new one started for the next analysis.
  cancel(): void {
    this.stop(new CancelledError());
  }

  private stop(reason: Error): void {
    this.worker?.terminate();
    this.worker = null;
    const pending = [...this.pending.values()];
    this.pending.clear();
    for (const p of pending) p.reject(reason);
  }
}
