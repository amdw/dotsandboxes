// Turning engine analysis results into what the interface shows: overall summaries
// and per-move labels for the board
import type { CPosMove, Evaluation, NimstringValue, PositionData } from "../engine/types";
import { isCoinCaptured, lineKey, moveToLine } from "./lines";
import { other, type Player } from "./state";

export type AnalysisKind = "evaluate" | "nimstring";

// What the per-move values on the board show after an evaluation: the engine's value of the
// rest of the game, or that plus the existing score difference (the final margin)
export type ValueMode = "engine" | "margin";

export interface MoveLabel {
  text: string;
  best: boolean;
}

// Labels for the board, keyed by lineKey
export type MoveLabels = Map<string, MoveLabel>;

export function positionKey(position: PositionData): string {
  return JSON.stringify(position);
}

export function lineKeyOf(move: CPosMove): string {
  return lineKey(moveToLine(move));
}

export function signed(n: number): string {
  return n > 0 ? `+${n}` : n < 0 ? `−${-n}` : "0";
}

// The symbol for a loony value, as in the paper (LaTeX \rightmoon) and the wider literature,
// with a variation selector asking for the text rather than the emoji form
export const LOONY = "\u263D\uFE0E";

export function formatNimstring(value: NimstringValue): string {
  return value === "Loony" ? LOONY : `*${value.Nimber}`;
}

export function uncapturedCoins(position: PositionData): number {
  let n = 0;
  for (const p of position.parts) {
    for (let y = 0; y < p.height; y++) {
      for (let x = 0; x < p.width; x++) {
        if (!isCoinCaptured(p, x, y)) n++;
      }
    }
  }
  return n;
}

export interface EvaluationSummary {
  mover: Player;
  // Net coins the player to move gains from the rest of the game with best play
  value: number;
  // The player to move's current lead
  scoreDiff: number;
  // value + scoreDiff: the player to move's final winning margin with best play
  margin: number;
  // Final scores with best play
  finalScores: Record<Player, number>;
}

export function summariseEvaluation(evaluation: Evaluation, position: PositionData, mover: Player, scores: Record<Player, number>): EvaluationSummary {
  const opponent = other(mover);
  const scoreDiff = scores[mover] - scores[opponent];
  const remaining = uncapturedCoins(position);
  // The mover takes (remaining + value) / 2 of the remaining coins, the opponent the rest
  const moverGain = (remaining + evaluation.value) / 2;
  return {
    mover,
    value: evaluation.value,
    scoreDiff,
    margin: evaluation.value + scoreDiff,
    finalScores: { [mover]: scores[mover] + moverGain, [opponent]: scores[opponent] + remaining - moverGain } as Record<Player, number>,
  };
}

export function evaluationLabels(evaluation: Evaluation, scoreDiff: number, mode: ValueMode): MoveLabels {
  const best = Math.max(...evaluation.moves.map((m) => m.value));
  return new Map(
    evaluation.moves.map((m) => [lineKeyOf(m.move), { text: signed(mode === "engine" ? m.value : m.value + scoreDiff), best: m.value === best }]),
  );
}

export function nimstringLabels(moves: { move: CPosMove; value: NimstringValue }[]): MoveLabels {
  return new Map(moves.map((m) => [lineKeyOf(m.move), { text: formatNimstring(m.value), best: false }]));
}
