import { beforeAll, describe, expect, it } from "vitest";
import { engine } from "../engine/engine";
import { allLines, isCoinCaptured, isEndOfGame, isStringPresent, lineToMove, moveToLine } from "./lines";
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
});
