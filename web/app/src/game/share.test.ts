import { beforeAll, describe, expect, it } from "vitest";
import { engine } from "../engine/engine";
import type { CPosMove } from "../engine/types";
import { decodeGame, encodeGame } from "./share";
import { current, goTo, newGameState, playMove, startFrom, startFromCurrent, startSplit, withStart, type GameState } from "./state";
import { loadTestEngine } from "./testEngine";

beforeAll(loadTestEngine);

const example = (id: string) => engine.examples().find((e) => e.id === id)!;

// Play some legal moves (the first legal move each time), as a game would
function playSome(game: GameState, count: number): GameState {
  for (let i = 0; i < count && !current(game).gameOver; i++) {
    game = playMove(game, engine.legalMoves(current(game).position)[0], engine.makeMove);
  }
  return game;
}

function roundTrip(game: GameState, view: "dab" | "snc" = "dab") {
  const fragment = encodeGame(game, view);
  const decoded = decodeGame(fragment, engine.makeMove);
  expect(decoded.game).toEqual(game);
  expect(decoded.view ?? "dab").toBe(view);
  return fragment;
}

describe("shareable links", () => {
  it("don't escape anything in links", () => {
    // A compound position (commas between parts) with base scores (a comma between them)
    const game = startFrom(current(newGameState(example("p45").position)), { baseScores: { A: 2, B: 3 }, voids: null });
    const fragment = roundTrip(game, "snc");
    expect(fragment).toMatch(/^[A-Za-z0-9_.,=&x-]+$/);
    expect(fragment).toContain("s=2,3");
  });

  it("encode a new game with just the position", () => {
    const fragment = roundTrip(newGameState(engine.newGame([[3, 3]])));
    expect(fragment).toMatch(/^p=3x3\.[A-Za-z0-9_-]+$/);
  });

  it("round-trip games with moves, an earlier move shown, and the view", () => {
    const game = playSome(newGameState(engine.newGame([[3, 3]])), 24);
    expect(current(game).gameOver).toBe(true);
    const fragment = roundTrip(game);
    // One character per move on a 3x3 board (24 lines)
    expect(new URLSearchParams(fragment).get("m")).toHaveLength(24);
    roundTrip(goTo(game, 5), "snc");
    roundTrip(goTo(game, 0));
  });

  it("round-trip game records and compound positions", () => {
    let record = newGameState(example("3x3-sample-game").position);
    for (const m of example("3x3-sample-game").moves) record = playMove(record, m, engine.makeMove);
    roundTrip(record);
    // Moves in several parts of P(4,5)
    let p45 = newGameState(example("p45").position);
    for (const part of [0, 3, 1]) {
      p45 = playMove(p45, engine.legalMoves(current(p45).position).find((m: CPosMove) => m.part === part)!, engine.makeMove);
    }
    roundTrip(p45);
  });

  it("use more characters per move on larger boards", () => {
    const game = playSome(newGameState(engine.newGame([[10, 10]])), 30);
    const fragment = roundTrip(game);
    // 220 lines need two characters per move
    expect(new URLSearchParams(fragment).get("m")).toHaveLength(60);
    roundTrip(playSome(newGameState(engine.newGame([[20, 20], [20, 20]])), 10));
  });

  it("round-trip split games, with owners, base scores, voids and B to move first", () => {
    // A takes a coin of an opened chain, then the game is split and continued
    let game = newGameState(example("double-deal").position);
    game = playMove(game, { part: 0, m: { x: 0, y: 0, side: "Left" } }, engine.makeMove);
    game = playMove(game, { part: 0, m: { x: 0, y: 0, side: "Right" } }, engine.makeMove);
    const split = startSplit(game, engine.split(current(game).position)!);
    expect(split.baseScores).toEqual({ A: 0, B: 1 });
    roundTrip(playSome(split, 3));

    // Voids, from splitting an L-shaped component
    const lShape = startSplit(newGameState(engine.parsePos("2 2\n1 0 b\n0 1 r\n1 1 b\n")), engine.split(engine.parsePos("2 2\n1 0 b\n0 1 r\n1 1 b\n"))!);
    expect(new URLSearchParams(encodeGame(lShape, "dab")).get("o")).toBe("...v.");
    roundTrip(lShape);

    // Owners at the start (from "Start from here"), and B to move first
    let owned = playSome(newGameState(engine.newGame([[2, 2]])), 4);
    owned = startFromCurrent(owned);
    roundTrip(owned);
    roundTrip(withStart(owned, { toMove: owned.snapshots[0].toMove === "A" ? "B" : "A" }, engine.makeMove));
  });

  it("reject broken links", () => {
    const good = encodeGame(playSome(newGameState(engine.newGame([[2, 2]])), 3), "dab");
    const bad = (change: (p: URLSearchParams) => void) => {
      const params = new URLSearchParams(good);
      change(params);
      return () => decodeGame(params.toString(), engine.makeMove);
    };
    expect(() => decodeGame("", engine.makeMove)).toThrow(/No position/);
    expect(bad((p) => p.set("p", "2x2"))).toThrow(/Invalid part/);
    expect(bad((p) => p.set("p", "0x2.AAA"))).toThrow(/Invalid part size/);
    expect(bad((p) => p.set("p", "2x2.A"))).toThrow(/Wrong number of strings/);
    expect(bad((p) => p.set("m", p.get("m")! + p.get("m")![0]))).toThrow(/Not a legal move/);
    expect(bad((p) => p.set("m", "_"))).toThrow(/Invalid move/);
    expect(bad((p) => p.set("c", "9"))).toThrow(/Invalid move number/);
    expect(bad((p) => p.set("o", "AB"))).toThrow(/Invalid owners/);
    expect(bad((p) => p.set("s", "x"))).toThrow(/Invalid scores/);
    expect(bad((p) => p.set("t", "C"))).toThrow(/Invalid first player/);
  });
});
