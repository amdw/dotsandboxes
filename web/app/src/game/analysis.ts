// Turning engine analysis results into what the interface shows: per-move labels
// for the board
import type { ComponentValue, CPosMove, Evaluation, NimstringValue, PositionData } from "../engine/types";
import { coinsCapturedBy, lineKey, moveToLine } from "./lines";

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

// Labels for the board from an evaluation (whose values are for the player to move). With
// flip, the values are negated, i.e. shown from the other player's point of view. Either way,
// the best moves (for the player to move) are marked.
export function evaluationLabels(evaluation: Evaluation, scoreDiff: number, mode: ValueMode, flip = false): MoveLabels {
  const best = Math.max(...evaluation.moves.map((m) => m.value));
  return new Map(
    evaluation.moves.map((m) => {
      const value = mode === "engine" ? m.value : m.value + scoreDiff;
      return [lineKeyOf(m.move), { text: signed(flip ? -value : value), best: m.value === best }];
    }),
  );
}

// Whether a move wins at Nimstring, given the value of the position after it. A move which
// doesn't capture passes the turn, so it wins if it leaves *0 for the opponent; a capture
// means moving again, so it wins if it leaves a non-zero value (including loony).
export function isWinningNimstringMove(captures: boolean, valueAfter: NimstringValue): boolean {
  const zero = valueAfter !== "Loony" && valueAfter.Nimber === 0;
  return captures ? !zero : zero;
}

// Labels for the board, with the winning moves marked as best
export function nimstringLabels(moves: { move: CPosMove; value: NimstringValue }[], position: PositionData): MoveLabels {
  return new Map(
    moves.map((m) => {
      const line = moveToLine(m.move);
      const captures = coinsCapturedBy(position.parts[line.part], line).length > 0;
      return [lineKey(line), { text: formatNimstring(m.value), best: isWinningNimstringMove(captures, m.value) }];
    }),
  );
}

// Nimstring sum: values add as nimbers, and a sum with any loony term is loony
export function nimAdd(a: NimstringValue, b: NimstringValue): NimstringValue {
  return a === "Loony" || b === "Loony" ? "Loony" : { Nimber: a.Nimber ^ b.Nimber };
}

// The Nimstring value of each part of a position, from the values of its components
// (a part with no strings left has value *0)
export function partValues(components: ComponentValue[], partCount: number): NimstringValue[] {
  const values: NimstringValue[] = Array.from({ length: partCount }, () => ({ Nimber: 0 }));
  for (const c of components) {
    values[c.part] = nimAdd(values[c.part], c.value);
  }
  return values;
}

// The move the engine plays: one of the best moves in an evaluation, chosen at random
// (random() returning a number in [0, 1)) so that different playouts can vary
export function chooseEngineMove(evaluation: Evaluation, random: () => number = Math.random): CPosMove | null {
  const best = evaluation.moves.filter((m) => m.value === evaluation.value);
  if (best.length === 0) return null;
  return best[Math.min(best.length - 1, Math.floor(random() * best.length))].move;
}

export type MoveMark = "?" | "??";

export interface MoveJudgement {
  // "?" for a move which wasn't optimal but didn't change the outcome (win, draw or loss),
  // "??" for one which did, or null for an optimal move
  mark: MoveMark | null;
  // Final margins for the player who moved, counting the score so far, after the best
  // move and after the move played
  best: number;
  played: number;
}

// Judge a move, given the evaluation of the position before it (for the player who moved)
// and that player's lead before it. Null if the move isn't one of the evaluation's.
export function judgeMove(before: Evaluation, move: CPosMove, scoreDiffBefore: number): MoveJudgement | null {
  const key = lineKeyOf(move);
  const played = before.moves.find((m) => lineKeyOf(m.move) === key);
  if (!played) return null;
  const best = before.value + scoreDiffBefore;
  const result = played.value + scoreDiffBefore;
  const mark = result === best ? null : Math.sign(result) !== Math.sign(best) ? "??" : "?";
  return { mark, best, played: result };
}
