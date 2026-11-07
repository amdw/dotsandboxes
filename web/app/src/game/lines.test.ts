import { beforeAll, describe, expect, it } from "vitest";
import { engine } from "../engine/engine";
import { allLines, coinsCapturedBy, isCoinCaptured, toggleString, isEndOfGame, isStringPresent, lineToMove, moveToLine } from "./lines";
import { loadTestEngine } from "./testEngine";

beforeAll(loadTestEngine);

describe("lines", () => {
  it("map to the engine's normal form of moves and back", () => {
    const position = engine.newGame([[3, 2]]);
    const part = position.parts[0];
    const lines = allLines(part, 0);
    expect(lines).toHaveLength(17);
    const moves = lines.map(lineToMove);
    const key = (m: { part: number; m: { x: number; y: number; side: string } }) => `${m.part} ${m.m.x} ${m.m.y} ${m.m.side}`;
    expect(new Set(moves.map(key))).toEqual(new Set(engine.legalMoves(position).map(key)));
    for (const line of lines) {
      expect(moveToLine(lineToMove(line))).toEqual(line);
    }
  });

  it("map non-normal moves to the same line", () => {
    expect(moveToLine({ part: 0, m: { x: 1, y: 0, side: "Left" } })).toEqual(moveToLine({ part: 0, m: { x: 0, y: 0, side: "Right" } }));
    expect(moveToLine({ part: 2, m: { x: 0, y: 1, side: "Top" } })).toEqual(moveToLine({ part: 2, m: { x: 0, y: 0, side: "Bottom" } }));
  });

  it("track strings and captured coins", () => {
    let position = engine.newGame([[2, 1]]);
    const left = { part: 0, orientation: "v" as const, x: 0, y: 0 };
    expect(isStringPresent(position.parts[0], left)).toBe(true);
    for (const line of [left, { part: 0, orientation: "h" as const, x: 0, y: 0 }, { part: 0, orientation: "h" as const, x: 0, y: 1 }]) {
      position = engine.makeMove(position, lineToMove(line)).position;
    }
    expect(isStringPresent(position.parts[0], left)).toBe(false);
    expect(isCoinCaptured(position.parts[0], 0, 0)).toBe(false);
    position = engine.makeMove(position, { part: 0, m: { x: 0, y: 0, side: "Right" } }).position;
    expect(isCoinCaptured(position.parts[0], 0, 0)).toBe(true);
    expect(isCoinCaptured(position.parts[0], 1, 0)).toBe(false);
    expect(isEndOfGame(position.parts)).toBe(false);
  });

  it("find the squares a line would complete", () => {
    // 2x1 board with everything drawn except the middle line and the right edge
    let position = engine.newGame([[2, 1]]);
    for (const m of engine.legalMoves(position)) {
      const right = m.m.x === 1 && m.m.side === "Right";
      const middle = m.m.x === 0 && m.m.side === "Right";
      if (!right && !middle) position = engine.makeMove(position, m).position;
    }
    const part = position.parts[0];
    const middle = { part: 0, orientation: "v" as const, x: 1, y: 0 };
    const right = { part: 0, orientation: "v" as const, x: 2, y: 0 };
    expect(coinsCapturedBy(part, middle)).toEqual([[0, 0]]);
    expect(coinsCapturedBy(part, right)).toEqual([]);
    // Once the right edge is drawn, the middle line completes both squares
    const after = engine.makeMove(position, lineToMove(right)).position.parts[0];
    expect(coinsCapturedBy(after, middle)).toEqual([[0, 0], [1, 0]]);
    // A line which has already been drawn completes nothing
    expect(coinsCapturedBy(after, right)).toEqual([]);
  });

  it("toggle single strings without affecting others or the original", () => {
    const position = engine.newGame([[2, 2], [1, 1]]);
    const line = { part: 1, orientation: "v" as const, x: 1, y: 0 };
    const edited = toggleString(position, line);
    expect(isStringPresent(edited.parts[1], line)).toBe(false);
    expect(isStringPresent(position.parts[1], line)).toBe(true);
    expect(edited.parts[0]).toBe(position.parts[0]);
    // The engine accepts the result, and agrees the string is gone
    expect(engine.legalMoves(edited)).toHaveLength(engine.legalMoves(position).length - 1);
    expect(toggleString(edited, line)).toEqual(position);
  });
});
