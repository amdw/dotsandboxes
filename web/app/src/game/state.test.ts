import { beforeAll, describe, expect, it } from "vitest";
import { engine } from "../engine/engine";
import type { CPosMove } from "../engine/types";
import { current, goTo, newGameState, nextOwner, playMove, startFrom, withStart } from "./state";
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

  it("starts a new game from the current position, keeping owners and the player to move", () => {
    const chain = engine.examples().find((e) => e.id === "nimstring-examples")!.position.parts[5]; // 3-chain
    let game = newGameState({ parts: [chain] });
    game = playMove(game, mv(0, 0, "Left"), engine.makeMove); // A opens
    game = playMove(game, mv(0, 0, "Right"), engine.makeMove); // B takes one, moves again
    const fresh = startFrom(current(game));
    expect(fresh.snapshots).toHaveLength(1);
    expect(current(fresh).toMove).toBe("B");
    expect(current(fresh).scores).toEqual({ A: 0, B: 1 });
    expect(current(fresh).owners[0][0]).toEqual(["B", null, null]);
    expect(current(fresh).lastMove).toBeNull();
  });

  it("cycles owners through A, B and nobody", () => {
    expect(nextOwner(null)).toBe("A");
    expect(nextOwner("A")).toBe("B");
    expect(nextOwner("B")).toBeNull();
  });

  it("replays later moves when the starting position is changed", () => {
    // Start with coin 0 of a 3-chain already captured by nobody, then play two moves
    const chain = engine.examples().find((e) => e.id === "nimstring-examples")!.position.parts[5];
    let game = newGameState({ parts: [chain] });
    game = playMove(game, mv(0, 0, "Left"), engine.makeMove);
    game = startFrom(playMove(game, mv(0, 0, "Right"), engine.makeMove).snapshots.at(-1)!);
    game = playMove(game, mv(1, 0, "Right"), engine.makeMove); // B takes another
    game = playMove(game, mv(2, 0, "Right"), engine.makeMove); // and the last
    game = goTo(game, 1);

    const owners = [[["A", null, null] as ("A" | "B" | null)[]]];
    const changed = withStart(game, { owners, toMove: "A" }, engine.makeMove);
    expect(changed.cursor).toBe(1);
    expect(changed.snapshots).toHaveLength(3);
    expect(changed.snapshots[0].scores).toEqual({ A: 1, B: 0 });
    // Now A makes the later captures, on top of the edited start
    expect(changed.snapshots[2].scores).toEqual({ A: 3, B: 0 });
    expect(changed.snapshots[2].owners[0][0]).toEqual(["A", "A", "A"]);
  });

  it("keeps a compound game going until every part is finished", () => {
    // Two parts, each a single coin with one string to the ground
    const coin = engine.examples().find((e) => e.id === "nimstring-examples")!.position.parts[0];
    let game = newGameState({ parts: [coin, coin] });
    game = playMove(game, mv(0, 0, "Left", 0), engine.makeMove);
    expect(current(game).gameOver).toBe(false);
    expect(current(game).toMove).toBe("A");
    game = playMove(game, mv(0, 0, "Left", 1), engine.makeMove);
    expect(current(game).gameOver).toBe(true);
    expect(current(game).scores).toEqual({ A: 2, B: 0 });
  });
});
