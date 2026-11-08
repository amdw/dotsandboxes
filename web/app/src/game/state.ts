// The state of a game being played through: a list of snapshots, one per move made
// from the starting position, and a cursor into it for moving back and forward.
import type { Coin, CPosMove, MoveResult, PositionData } from "../engine/types";
import { isCoinCaptured, isEndOfGame } from "./lines";

export type Player = "A" | "B";

export interface Snapshot {
  position: PositionData;
  toMove: Player;
  scores: Record<Player, number>;
  // owners[part][y][x]: who captured each coin, or null if it is uncaptured
  // or was already captured in the starting position
  owners: (Player | null)[][][];
  lastMove: CPosMove | null;
  lastCaptured: Coin[];
  gameOver: boolean;
}

export interface GameState {
  snapshots: Snapshot[];
  cursor: number;
  // Points scored before the starting position, other than for the coins on the board
  // with owners (e.g. for coins dropped when the position was split)
  baseScores: Record<Player, number>;
  // voids[part][y][x]: cells which aren't coins at all, but just fill out a part to a
  // rectangle (e.g. after splitting a position into its components), or null if none
  voids: boolean[][][] | null;
}

// Settings of a game which apply from its starting position onwards
type StartExtras = Pick<GameState, "baseScores" | "voids">;

const NO_EXTRAS: StartExtras = { baseScores: { A: 0, B: 0 }, voids: null };

export type MakeMove = (position: PositionData, move: CPosMove) => MoveResult;

export function newGameState(position: PositionData): GameState {
  const owners = position.parts.map((p) => Array.from({ length: p.height }, () => Array<Player | null>(p.width).fill(null)));
  return {
    snapshots: [
      {
        position,
        toMove: "A",
        scores: { A: 0, B: 0 },
        owners,
        lastMove: null,
        lastCaptured: [],
        gameOver: isEndOfGame(position.parts),
      },
    ],
    cursor: 0,
    ...NO_EXTRAS,
  };
}

export function current(game: GameState): Snapshot {
  return game.snapshots[game.cursor];
}

// Make a move from the current snapshot, discarding any later snapshots
export function playMove(game: GameState, move: CPosMove, makeMove: MakeMove): GameState {
  const before = current(game);
  const result = makeMove(before.position, move);
  const mover = before.toMove;
  const owners = before.owners.map((part) => part.map((row) => row.slice()));
  for (const coin of result.captured) {
    owners[coin.part][coin.y][coin.x] = mover;
  }
  const after: Snapshot = {
    position: result.position,
    toMove: result.coins_captured > 0 ? mover : other(mover),
    scores: { ...before.scores, [mover]: before.scores[mover] + result.coins_captured },
    owners,
    lastMove: result.move,
    lastCaptured: result.captured,
    gameOver: result.end_of_game,
  };
  return { ...game, snapshots: [...game.snapshots.slice(0, game.cursor + 1), after], cursor: game.cursor + 1 };
}

export function other(player: Player): Player {
  return player === "A" ? "B" : "A";
}

export function goTo(game: GameState, cursor: number): GameState {
  return { ...game, cursor: Math.max(0, Math.min(game.snapshots.length - 1, cursor)) };
}

export function scoresOf(owners: (Player | null)[][][]): Record<Player, number> {
  const scores = { A: 0, B: 0 };
  for (const owner of owners.flat(2)) {
    if (owner) scores[owner]++;
  }
  return scores;
}

// The owner a captured coin gets when clicked while setting up a starting position
export function nextOwner(owner: Player | null): Player | null {
  return owner === "A" ? "B" : owner === "B" ? null : "A";
}

// A new game starting from a snapshot (e.g. the current position of another game),
// keeping its coin owners, scores and player to move
export function startFrom(snapshot: Snapshot, extras: StartExtras = NO_EXTRAS): GameState {
  const owned = scoresOf(snapshot.owners);
  const scores = { A: extras.baseScores.A + owned.A, B: extras.baseScores.B + owned.B };
  return {
    snapshots: [{ ...snapshot, scores, lastMove: null, lastCaptured: [] }],
    cursor: 0,
    ...extras,
  };
}

// A new game starting from the current position of a game
export function startFromCurrent(game: GameState): GameState {
  return startFrom(current(game), { baseScores: game.baseScores, voids: game.voids });
}

// A new game starting from the current position of a game split into its independent
// components (see engine.split), keeping the scores and player to move. Captured coins
// aren't part of the split position, so the scores so far become base scores, and any
// cells without strings are just padding, not coins.
export function startSplit(game: GameState, split: PositionData): GameState {
  const snapshot = current(game);
  const owners = split.parts.map((p) => Array.from({ length: p.height }, () => Array<Player | null>(p.width).fill(null)));
  const voids = split.parts.map((p) => Array.from({ length: p.height }, (_, y) => Array.from({ length: p.width }, (_, x) => isCoinCaptured(p, x, y))));
  return startFrom(
    { ...snapshot, position: split, owners, gameOver: isEndOfGame(split.parts) },
    { baseScores: snapshot.scores, voids: voids.some((part) => part.some((row) => row.some((v) => v))) ? voids : null },
  );
}

// Change the coin owners and/or player to move of the starting position, replaying
// any later moves so their scores and turns stay consistent with it
export function withStart(
  game: GameState,
  start: { owners?: (Player | null)[][][]; toMove?: Player },
  makeMove: MakeMove,
): GameState {
  let result = startFrom({ ...game.snapshots[0], ...start }, { baseScores: game.baseScores, voids: game.voids });
  for (const snapshot of game.snapshots.slice(1)) {
    result = playMove(result, snapshot.lastMove!, makeMove);
  }
  return goTo(result, game.cursor);
}

// A new game with just one part of the current position of a game, keeping that
// part's coin owners (and so their points) and the player to move
export function isolatePart(game: GameState, part: number): GameState {
  const snapshot = current(game);
  const position = { parts: [snapshot.position.parts[part]] };
  return startFrom(
    { ...snapshot, position, owners: [snapshot.owners[part]], gameOver: isEndOfGame(position.parts) },
    { baseScores: { A: 0, B: 0 }, voids: game.voids ? [game.voids[part]] : null },
  );
}

// The same game started again from its starting position, with the moves made since
// discarded (keeping the starting owners, scores, player to move and voids)
export function restart(game: GameState): GameState {
  return startFrom(game.snapshots[0], { baseScores: game.baseScores, voids: game.voids });
}

// A sequence of canonical moves, and the coins they capture
export interface CanonicalSequence {
  moves: CPosMove[];
  captured: Coin[];
}

// The canonical moves (given by canonicalMoves) which can be played one after another
// from a position until there are none left. They're all captures, so all by the player
// to move, and all on strings present in the position.
export function canonicalSequence(position: PositionData, canonicalMoves: (position: PositionData) => CPosMove[], makeMove: MakeMove): CanonicalSequence {
  const sequence: CanonicalSequence = { moves: [], captured: [] };
  let current = position;
  for (let i = 0; i < 10000; i++) {
    const moves = canonicalMoves(current);
    if (moves.length === 0) break;
    sequence.moves.push(moves[0]);
    const result = makeMove(current, moves[0]);
    sequence.captured.push(...result.captured);
    if (result.end_of_game) break;
    current = result.position;
  }
  return sequence;
}

// Play the canonical sequence from the current position, each as a separate move in the history
export function playCanonicalMoves(game: GameState, canonicalMoves: (position: PositionData) => CPosMove[], makeMove: MakeMove): GameState {
  return canonicalSequence(current(game).position, canonicalMoves, makeMove).moves.reduce((g, m) => playMove(g, m, makeMove), game);
}
