import { beforeAll, describe, expect, it } from "vitest";
import { engine } from "../engine/engine";
import type { CPosMove } from "../engine/types";
import { canonicalSequence, current, goTo, isolatePart, newGameState, nextOwner, playCanonicalMoves, playMove, restart, startFrom, startFromCurrent, startSplit, withStart } from "./state";
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

  it("splits a game into its components, keeping scores and the player to move", () => {
    // Two 5-chains in one part; A opens one and B takes a coin, so must move again
    const position = engine.examples().find((e) => e.id === "double-deal")!.position; // two 5-chains
    let game = newGameState(position);
    game = playMove(game, mv(0, 0, "Left"), engine.makeMove); // A opens the top chain
    game = playMove(game, mv(0, 0, "Right"), engine.makeMove); // B takes a coin
    const split = engine.split(current(game).position)!;
    expect(split.parts).toHaveLength(2);
    const fresh = startSplit(game, split);
    expect(fresh.snapshots).toHaveLength(1);
    expect(current(fresh).toMove).toBe("B");
    expect(current(fresh).scores).toEqual({ A: 0, B: 1 });
    expect(fresh.baseScores).toEqual({ A: 0, B: 1 });
    // The captured coin is gone; the rest of the top chain is a 4-coin part, with no voids
    expect(split.parts.map((p) => p.width).sort()).toEqual([4, 5]);
    expect(fresh.voids).toBeNull();

    // Scores carry on from the base, and survive starting again or changing the start
    // B takes the next coin of the opened chain (its left string is already cut)
    let later = playMove(fresh, mv(0, 0, "Right", split.parts.findIndex((p) => p.width === 4)), engine.makeMove);
    expect(current(later).scores).toEqual({ A: 0, B: 2 });
    later = startFromCurrent(later);
    expect(current(later).scores).toEqual({ A: 0, B: 2 });
    expect(later.voids).toBe(fresh.voids);
    const switched = withStart(fresh, { toMove: "A" }, engine.makeMove);
    expect(current(switched).scores).toEqual({ A: 0, B: 1 });
  });

  it("marks cells which aren't coins after splitting", () => {
    // A 2x2 board where the bottom-right coin is cut off from the rest, with just a
    // string to the ground on its right: an L-shaped component, and a single coin
    const position = engine.parsePos("2 2\n1 0 b\n0 1 r\n1 1 b\n");
    const split = engine.split(position)!;
    expect(split.parts.map((p) => [p.width, p.height])).toEqual([[2, 2], [1, 1]]);
    const fresh = startSplit(newGameState(position), split);
    // The L-shaped part's bounding box includes a cell which isn't one of its coins
    expect(fresh.voids).toEqual([
      [
        [false, false],
        [false, true],
      ],
      [[false]],
    ]);
    // Voids don't score
    expect(current(fresh).scores).toEqual({ A: 0, B: 0 });
  });

  it("isolates one part of a game, keeping its owners and voids but not other points", () => {
    // A single coin and a 3-chain as separate parts, each with a coin captured
    const coin = engine.examples().find((e) => e.id === "nimstring-examples")!.position.parts[0];
    const chain = engine.examples().find((e) => e.id === "nimstring-examples")!.position.parts[5]; // 3-chain
    let game = newGameState({ parts: [coin, chain] });
    game = playMove(game, mv(0, 0, "Left", 0), engine.makeMove); // A takes part 0's coin
    game = playMove(game, mv(0, 0, "Left", 1), engine.makeMove); // A opens the chain
    game = playMove(game, mv(0, 0, "Right", 1), engine.makeMove); // B takes a coin of it
    expect(current(game).scores).toEqual({ A: 1, B: 1 });

    const chainOnly = isolatePart(game, 1);
    expect(chainOnly.snapshots).toHaveLength(1);
    expect(current(chainOnly).position.parts).toEqual([current(game).position.parts[1]]);
    expect(current(chainOnly).owners).toEqual([[["B", null, null]]]);
    expect(current(chainOnly).scores).toEqual({ A: 0, B: 1 });
    expect(current(chainOnly).toMove).toBe("B");
    expect(chainOnly.voids).toBeNull();

    // Voids from a split go with their part, but the split's base scores (here A 1, B 1,
    // carried over from the game above) don't
    const split = startSplit(game, engine.split(engine.parsePos("2 2\n1 0 b\n0 1 r\n1 1 b\n"))!);
    expect(split.baseScores).toEqual({ A: 1, B: 1 });
    const lShape = isolatePart(split, 0);
    expect(lShape.voids).toEqual([
      [
        [false, false],
        [false, true],
      ],
    ]);
    expect(current(lShape).scores).toEqual({ A: 0, B: 0 });
  });

  it("restarts a game from its starting position", () => {
    // A split game (with base scores) where B is to move, played on a little
    let game = newGameState(engine.examples().find((e) => e.id === "double-deal")!.position);
    game = playMove(game, mv(0, 0, "Left"), engine.makeMove);
    game = playMove(game, mv(0, 0, "Right"), engine.makeMove);
    game = startSplit(game, engine.split(current(game).position)!);
    const start = current(game);
    game = playMove(game, mv(0, 0, "Right", 0), engine.makeMove);
    game = playMove(game, mv(1, 0, "Right", 0), engine.makeMove);
    game = goTo(game, 1);

    const restarted = restart(game);
    expect(restarted.snapshots).toHaveLength(1);
    expect(current(restarted)).toEqual(start);
    expect(restarted.baseScores).toEqual(game.baseScores);
    expect(restarted.voids).toBe(game.voids);
  });

  it("plays canonical moves until the decision point", () => {
    // A opens one of two 5-chains; B takes all but the last two coins, then must decide
    // whether to take them too or double-deal
    let game = newGameState(engine.examples().find((e) => e.id === "double-deal")!.position);
    game = playMove(game, mv(0, 0, "Left"), engine.makeMove);
    expect(engine.canonicalMoves(current(game).position)).toHaveLength(1);
    game = playCanonicalMoves(game, engine.canonicalMoves, engine.makeMove);
    expect(game.snapshots).toHaveLength(5);
    expect(current(game).scores).toEqual({ A: 0, B: 3 });
    expect(current(game).toMove).toBe("B");
    expect(engine.canonicalMoves(current(game).position)).toEqual([]);
    // With nothing canonical, nothing happens
    expect(playCanonicalMoves(game, engine.canonicalMoves, engine.makeMove)).toBe(game);
  });

  it("finds the whole canonical sequence, including taking the last chain", () => {
    // Opened 5-chain with another 5-chain left: three coins, then the decision point
    let game = newGameState(engine.examples().find((e) => e.id === "double-deal")!.position);
    game = playMove(game, mv(0, 0, "Left"), engine.makeMove);
    const sequence = canonicalSequence(current(game).position, engine.canonicalMoves, engine.makeMove);
    expect(sequence).toHaveLength(3);
    expect(new Set(sequence.map((m) => `${m.m.x} ${m.m.y} ${m.m.side}`)).size).toBe(3);

    // B double-deals; now A takes the two coins, then the last chain is all canonical
    game = playCanonicalMoves(game, engine.canonicalMoves, engine.makeMove);
    game = playMove(game, mv(4, 0, "Right"), engine.makeMove); // double-deal: leave the last two
    game = playCanonicalMoves(game, engine.canonicalMoves, engine.makeMove); // A takes them...
    expect(current(game).scores).toEqual({ A: 2, B: 3 });
    // ...and must open the last chain; B can then take all of it
    game = playMove(game, mv(0, 1, "Left"), engine.makeMove);
    expect(canonicalSequence(current(game).position, engine.canonicalMoves, engine.makeMove)).toHaveLength(5);
    game = playCanonicalMoves(game, engine.canonicalMoves, engine.makeMove);
    expect(current(game).gameOver).toBe(true);
    expect(current(game).scores).toEqual({ A: 2, B: 8 });
  });
});

