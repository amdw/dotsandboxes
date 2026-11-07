import { beforeAll, describe, expect, it } from "vitest";
import { engine } from "../engine/engine";
import type { CPosMove } from "../engine/types";
import { current, goTo, newGameState, playMove } from "./state";
import { loadTestEngine } from "./testEngine";

beforeAll(loadTestEngine);

const mv = (x: number, y: number, side: CPosMove["m"]["side"], part = 0): CPosMove => ({ part, m: { x, y, side } });

describe("game state", () => {
  it("scores captures and gives the capturer another move", () => {
    // A 2-chain: A opens it, B takes both coins
    const chain = engine.examples().find((e) => e.id === "nimstring-examples")!.position.parts[4];
    let game = newGameState({ parts: [chain] });
    expect(current(game).gameOver).toBe(false);
    game = playMove(game, mv(0, 0, "Left"), engine.makeMove);
    expect(current(game).toMove).toBe("B");
    game = playMove(game, mv(1, 0, "Left"), engine.makeMove);
    expect(current(game).toMove).toBe("B");
    expect(current(game).scores).toEqual({ A: 0, B: 1 });
    expect(current(game).owners[0][0]).toEqual(["B", null]);
    expect(current(game).lastCaptured).toEqual([{ part: 0, x: 0, y: 0 }]);
    game = playMove(game, mv(1, 0, "Right"), engine.makeMove);
    expect(current(game).scores).toEqual({ A: 0, B: 2 });
    expect(current(game).gameOver).toBe(true);
  });

  it("plays through the sample 2x2 game from the paper", () => {
    const example = engine.examples().find((e) => e.id === "sample-game-2x2")!;
    let game = newGameState(example.position);
    for (const m of example.moves) {
      game = playMove(game, m, engine.makeMove);
    }
    const end = current(game);
    expect(end.gameOver).toBe(true);
    expect(end.scores.A + end.scores.B).toBe(4);
  });

  it("moves back and forward, and truncates history when moving from earlier", () => {
    let game = newGameState(engine.newGame([[2, 2]]));
    game = playMove(game, mv(0, 0, "Top"), engine.makeMove);
    game = playMove(game, mv(1, 0, "Top"), engine.makeMove);
    expect(game.snapshots).toHaveLength(3);
    game = goTo(game, 1);
    expect(current(game).lastMove).toEqual(mv(0, 0, "Top"));
    expect(goTo(game, -5).cursor).toBe(0);
    expect(goTo(game, 99).cursor).toBe(2);
    game = playMove(game, mv(0, 1, "Bottom"), engine.makeMove);
    expect(game.snapshots).toHaveLength(3);
    expect(game.cursor).toBe(2);
    expect(current(game).lastMove).toEqual(mv(0, 1, "Bottom"));
  });

  it("reports engine errors for illegal moves", () => {
    const game = newGameState(engine.newGame([[1, 1]]));
    const after = playMove(game, mv(0, 0, "Top"), engine.makeMove);
    expect(() => playMove(after, mv(0, 0, "Top"), engine.makeMove)).toThrow(/Not a legal move/);
  });
});
