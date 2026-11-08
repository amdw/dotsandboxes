// Turning engine analysis results into what the interface shows: per-move labels
// for the board
import type { ComponentValue, CPosMove, Evaluation, NimstringValue, PositionData } from "../engine/types";
import { lineKey, moveToLine } from "./lines";

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

export function evaluationLabels(evaluation: Evaluation, scoreDiff: number, mode: ValueMode): MoveLabels {
  const best = Math.max(...evaluation.moves.map((m) => m.value));
  return new Map(
    evaluation.moves.map((m) => [lineKeyOf(m.move), { text: signed(mode === "engine" ? m.value : m.value + scoreDiff), best: m.value === best }]),
  );
}

export function nimstringLabels(moves: { move: CPosMove; value: NimstringValue }[]): MoveLabels {
  return new Map(moves.map((m) => [lineKeyOf(m.move), { text: formatNimstring(m.value), best: false }]));
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
