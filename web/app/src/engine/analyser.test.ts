import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { Analyser, CancelledError, TimeLimitError, type AnalysisRequest, type AnalysisResponse, type WorkerMessage } from "./analyser";
import type { PositionData } from "./types";

// A stand-in for the analysis worker, recording the messages sent to it, and replying as told
class FakeWorker {
  static created: FakeWorker[] = [];
  received: WorkerMessage[] = [];
  terminated = false;
  onmessage: ((event: MessageEvent<AnalysisResponse>) => void) | null = null;
  onerror: ((event: ErrorEvent) => void) | null = null;

  constructor() {
    FakeWorker.created.push(this);
  }

  postMessage(message: WorkerMessage) {
    this.received.push(message);
  }

  terminate() {
    this.terminated = true;
  }

  reply(response: AnalysisResponse) {
    this.onmessage?.({ data: response } as MessageEvent<AnalysisResponse>);
  }

  requests(): AnalysisRequest[] {
    return this.received.filter((m): m is AnalysisRequest => m.type === "analyse");
  }
}

const position: PositionData = { parts: [] } as unknown as PositionData;

describe("Analyser", () => {
  beforeEach(() => {
    FakeWorker.created = [];
    vi.stubGlobal("Worker", FakeWorker);
  });
  afterEach(() => vi.unstubAllGlobals());

  it("passes on results and progress", async () => {
    const analyser = new Analyser();
    const progress: number[] = [];
    const result = analyser.run("evaluate", position, { onProgress: (p) => progress.push(p) });
    const worker = FakeWorker.created[0];
    const { id } = worker.requests()[0];
    worker.reply({ type: "progress", id, positions: 1000 });
    worker.reply({ type: "result", id, ok: true, result: null });
    await expect(result).resolves.toBeNull();
    expect(progress).toEqual([1000]);
  });

  it("passes on the time limit, and reports running out of time", async () => {
    const analyser = new Analyser();
    const result = analyser.run("evaluate", position, { timeLimit: 60 });
    const worker = FakeWorker.created[0];
    const [request] = worker.requests();
    expect(request.timeLimit).toBe(60);
    worker.reply({ type: "result", id: request.id, ok: false, error: "time limit" });
    const error = await result.catch((e: unknown) => e);
    expect(error).toBeInstanceOf(TimeLimitError);
    expect((error as TimeLimitError).seconds).toBe(60);
    expect(worker.terminated).toBe(false);
  });

  it("replaces the worker when the engine crashes", async () => {
    const analyser = new Analyser();
    const first = analyser.run("evaluate", position);
    const second = analyser.run("nimstring", position, { maxNew: 10 });
    const worker = FakeWorker.created[0];
    const [request] = worker.requests();
    worker.reply({ type: "result", id: request.id, ok: false, error: "The engine stopped unexpectedly", fatal: true });
    // Everything waiting for the crashed engine fails, not just the analysis it was doing
    await expect(first).rejects.toThrow("The engine stopped unexpectedly");
    await expect(second).rejects.toThrow("The engine stopped unexpectedly");
    expect(worker.terminated).toBe(true);
    // The next analysis gets a new worker
    void analyser.run("evaluate", position);
    expect(FakeWorker.created).toHaveLength(2);
    expect(FakeWorker.created[1].requests()).toHaveLength(1);
  });

  it("replaces the worker if the engine crashes in a cancelled analysis", async () => {
    const analyser = new Analyser();
    const cancelled = analyser.run("evaluate", position);
    analyser.cancel();
    await expect(cancelled).rejects.toBeInstanceOf(CancelledError);
    const worker = FakeWorker.created[0];
    worker.reply({ type: "result", id: worker.requests()[0].id, ok: false, error: "crashed", fatal: true });
    expect(worker.terminated).toBe(true);
  });

  it("keeps the worker after an ordinary error", async () => {
    const analyser = new Analyser();
    const result = analyser.run("evaluate", position);
    const worker = FakeWorker.created[0];
    worker.reply({ type: "result", id: worker.requests()[0].id, ok: false, error: "Not a legal position" });
    await expect(result).rejects.toThrow("Not a legal position");
    expect(worker.terminated).toBe(false);
    void analyser.run("evaluate", position);
    expect(FakeWorker.created).toHaveLength(1);
  });
});
