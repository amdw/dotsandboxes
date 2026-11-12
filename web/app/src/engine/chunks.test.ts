import { describe, expect, it } from "vitest";
import { CHUNK_POSITIONS, chunkPositions, MAX_CHUNK_POSITIONS } from "./chunks";

describe("chunkPositions", () => {
  it("starts with the smallest chunk", () => {
    expect(chunkPositions(0, 0)).toBe(CHUNK_POSITIONS);
    expect(chunkPositions(0, 0, 60_000)).toBe(CHUNK_POSITIONS);
  });

  it("grows chunks with the work done so far", () => {
    expect(chunkPositions(1_600_000_000, 1_000_000)).toBe(100_000_000);
    expect(chunkPositions(1e12, 1e9)).toBe(MAX_CHUNK_POSITIONS);
  });

  it("keeps chunks within the time left", () => {
    // 1M positions per second, with 2 seconds left: 2M rather than 1/16 of 160M
    expect(chunkPositions(160_000_000, 160_000, 162_000)).toBe(2_000_000);
    // With plenty of time left, the time limit makes no difference
    expect(chunkPositions(160_000_000, 160_000, 1_000_000)).toBe(10_000_000);
    // But never less than the smallest chunk, even when out of time
    expect(chunkPositions(160_000_000, 170_000, 162_000)).toBe(CHUNK_POSITIONS);
  });
});
