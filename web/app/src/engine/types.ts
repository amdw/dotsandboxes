// Types of the values exchanged with the WebAssembly engine (see web/engine-wasm/src/api.rs)

export type Side = "Top" | "Bottom" | "Left" | "Right";

// A move within one part of a position: square (x, y) and the side of it to cut
export interface Move {
  x: number;
  y: number;
  side: Side;
}

export interface CPosMove {
  part: number;
  m: Move;
}

// One rectangular part of a position. h_strings[y][x] (y in 0..=height) is the string
// crossed by the horizontal line along the top of square (x, y); v_strings[y][x]
// (x in 0..=width) the one crossed by the vertical line along its left. true means the
// string is still present, i.e. the line has not been drawn.
export interface PartData {
  width: number;
  height: number;
  h_strings: boolean[][];
  v_strings: boolean[][];
}

export interface PositionData {
  parts: PartData[];
}

export interface Coin {
  part: number;
  x: number;
  y: number;
}

export interface MoveResult {
  move: CPosMove;
  position: PositionData;
  coins_captured: number;
  captured: Coin[];
  end_of_turn: boolean;
  end_of_game: boolean;
}

export type NimstringValue = { Nimber: number } | "Loony";

export interface MoveValue<V> {
  move: CPosMove;
  value: V;
}

export interface Evaluation {
  value: number;
  moves: MoveValue<number>[];
}

export interface ComponentValue {
  part: number;
  coins: [number, number][];
  value: NimstringValue;
}

export interface NimstringAnalysis {
  value: NimstringValue;
  components: ComponentValue[];
  moves: MoveValue<NimstringValue>[];
}

export interface Example {
  id: string;
  title: string;
  description: string;
  position: PositionData;
  moves: CPosMove[];
}
