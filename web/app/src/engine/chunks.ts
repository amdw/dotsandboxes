// How much of a full analysis the worker does in each chunk (see worker.ts)

// New positions per chunk: at least this (well under a second's work), and at least
// 1/CHUNK_GROWTH of the positions analysed so far (so cancelling takes effect after at most
// that fraction of the time taken so far)
export const CHUNK_POSITIONS = 500_000;
export const CHUNK_GROWTH = 16;
// The most new positions that can be asked for at once (the engine takes a 32-bit count)
export const MAX_CHUNK_POSITIONS = 2 ** 32 - 1;

// The new positions for the next chunk of an analysis which has analysed `done` positions in
// `elapsedMs` so far. With a time limit, a chunk is no longer than the time left at the rate
// so far (but at least CHUNK_POSITIONS), so the limit is overrun by at most about that much.
export function chunkPositions(done: number, elapsedMs: number, timeLimitMs?: number): number {
  let chunk = Math.max(CHUNK_POSITIONS, Math.floor(done / CHUNK_GROWTH));
  if (timeLimitMs !== undefined && done > 0 && elapsedMs > 0) {
    const inTimeLeft = Math.floor((done / elapsedMs) * Math.max(0, timeLimitMs - elapsedMs));
    chunk = Math.max(CHUNK_POSITIONS, Math.min(chunk, inTimeLeft));
  }
  return Math.min(chunk, MAX_CHUNK_POSITIONS);
}
