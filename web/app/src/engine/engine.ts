// Typed access to the WebAssembly engine. The module must be initialised
// (with loadEngine, or initEngineSync in tests) before any other call.
import init, * as wasm from "../wasm/dabengine_wasm.js";
import type {
  CPosMove,
  Evaluation,
  Example,
  MoveResult,
  NimstringAnalysis,
  PositionData,
} from "./types";

export async function loadEngine(): Promise<void> {
  await init();
}

export function initEngineSync(module: BufferSource): void {
  wasm.initSync({ module });
}

export const engine = {
  newGame: (sizes: [number, number][]): PositionData => wasm.newGame(sizes),
  legalMoves: (position: PositionData): CPosMove[] => wasm.legalMoves(position),
  makeMove: (position: PositionData, move: CPosMove): MoveResult => wasm.makeMove(position, move),
  evaluate: (position: PositionData): Evaluation => wasm.evaluate(position),
  nimstringAnalysis: (position: PositionData): NimstringAnalysis => wasm.nimstringAnalysis(position),
  parsePos: (text: string): PositionData => wasm.parsePos(text),
  writePos: (position: PositionData): string => wasm.writePos(position),
  // The captures which can be made without loss (see engine eval::canonical_moves)
  canonicalMoves: (position: PositionData): CPosMove[] => wasm.canonicalMoves(position),
  // The position split into its independent components, or null if there is nothing to split
  split: (position: PositionData): PositionData | null => wasm.split(position) ?? null,
  examples: (): Example[] => wasm.examples(),
};

export type Engine = typeof engine;

// The message of an error thrown by the engine (or anything else), without an "Error: " prefix
export function errorMessage(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}
