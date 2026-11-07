// The state of a game being played through: a list of snapshots, one per move made
// from the starting position, and a cursor into it for moving back and forward.
import type { Coin, CPosMove, MoveResult, PositionData } from "../engine/types";
import { isEndOfGame } from "./lines";

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
}

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
  return { snapshots: [...game.snapshots.slice(0, game.cursor + 1), after], cursor: game.cursor + 1 };
}

export function other(player: Player): Player {
  return player === "A" ? "B" : "A";
}

export function goTo(game: GameState, cursor: number): GameState {
  return { ...game, cursor: Math.max(0, Math.min(game.snapshots.length - 1, cursor)) };
}
