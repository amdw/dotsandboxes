// Runs engine analyses in a Web Worker, one at a time, with cancellation
import type { AnalysisKind } from "../game/analysis";
import type { Evaluation, NimstringAnalysis, PositionData } from "./types";

export interface AnalysisRequest {
  id: number;
  kind: AnalysisKind;
  position: PositionData;
}

export type AnalysisResponse = { id: number; ok: true; result: Evaluation | NimstringAnalysis } | { id: number; ok: false; error: string };

export class CancelledError extends Error {
  constructor() {
    super("Analysis cancelled");
  }
}

export class Analyser {
  private worker: Worker | null = null;
  private nextId = 0;
  private pending: { id: number; resolve: (r: Evaluation | NimstringAnalysis) => void; reject: (e: Error) => void } | null = null;

  private getWorker(): Worker {
    if (!this.worker) {
      this.worker = new Worker(new URL("./worker.ts", import.meta.url), { type: "module" });
      this.worker.onmessage = (event: MessageEvent<AnalysisResponse>) => {
        const response = event.data;
        const pending = this.pending;
        if (!pending || pending.id !== response.id) return;
        this.pending = null;
        if (response.ok) pending.resolve(response.result);
        else pending.reject(new Error(response.error));
      };
      this.worker.onerror = (event) => {
        const pending = this.pending;
        this.pending = null;
        // The worker may be unusable (e.g. out of memory), so start a fresh one next time
        this.worker?.terminate();
        this.worker = null;
        pending?.reject(new Error(event.message || "The engine stopped unexpectedly"));
      };
    }
    return this.worker;
  }

  evaluate(position: PositionData): Promise<Evaluation> {
    return this.run("evaluate", position) as Promise<Evaluation>;
  }

  nimstringAnalysis(position: PositionData): Promise<NimstringAnalysis> {
    return this.run("nimstring", position) as Promise<NimstringAnalysis>;
  }

  private run(kind: AnalysisKind, position: PositionData): Promise<Evaluation | NimstringAnalysis> {
    this.cancel();
    const id = this.nextId++;
    return new Promise((resolve, reject) => {
      this.pending = { id, resolve, reject };
      const request: AnalysisRequest = { id, kind, position };
      this.getWorker().postMessage(request);
    });
  }

  // Stop any analysis in progress. The engine can't be interrupted, so the worker is
  // terminated, and a new one started for the next analysis.
  cancel(): void {
    if (this.pending) {
      this.pending.reject(new CancelledError());
      this.pending = null;
      this.worker?.terminate();
      this.worker = null;
    }
  }
}
