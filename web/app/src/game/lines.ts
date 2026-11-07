// Mapping between engine moves and the lines/strings of a part, as indexed in PartData
import type { CPosMove, PartData } from "../engine/types";

// A horizontal line at row y (0..=height), column x (0..width), or
// a vertical line at row y (0..height), column x (0..=width)
export interface Line {
  part: number;
  orientation: "h" | "v";
  x: number;
  y: number;
}

export function lineToMove(line: Line): CPosMove {
  const { part, x, y } = line;
  if (line.orientation === "h") {
    return y === 0 ? { part, m: { x, y: 0, side: "Top" } } : { part, m: { x, y: y - 1, side: "Bottom" } };
  }
  return x === 0 ? { part, m: { x: 0, y, side: "Left" } } : { part, m: { x: x - 1, y, side: "Right" } };
}

export function moveToLine(move: CPosMove): Line {
  const { part, m } = move;
  switch (m.side) {
    case "Top":
      return { part, orientation: "h", x: m.x, y: m.y };
    case "Bottom":
      return { part, orientation: "h", x: m.x, y: m.y + 1 };
    case "Left":
      return { part, orientation: "v", x: m.x, y: m.y };
    case "Right":
      return { part, orientation: "v", x: m.x + 1, y: m.y };
  }
}

export function sameLine(a: Line | null, b: Line | null): boolean {
  return a !== null && b !== null && a.part === b.part && a.orientation === b.orientation && a.x === b.x && a.y === b.y;
}

export function isStringPresent(data: PartData, line: Line): boolean {
  return line.orientation === "h" ? data.h_strings[line.y][line.x] : data.v_strings[line.y][line.x];
}

// Every line of a part, horizontal ones first
export function allLines(data: PartData, part: number): Line[] {
  const lines: Line[] = [];
  for (let y = 0; y <= data.height; y++) {
    for (let x = 0; x < data.width; x++) {
      lines.push({ part, orientation: "h", x, y });
    }
  }
  for (let y = 0; y < data.height; y++) {
    for (let x = 0; x <= data.width; x++) {
      lines.push({ part, orientation: "v", x, y });
    }
  }
  return lines;
}

// A coin is captured once all four of its strings have been cut
export function isCoinCaptured(data: PartData, x: number, y: number): boolean {
  return !(data.h_strings[y][x] || data.h_strings[y + 1][x] || data.v_strings[y][x] || data.v_strings[y][x + 1]);
}

export function isEndOfGame(parts: PartData[]): boolean {
  return parts.every((p) => p.h_strings.every((row) => row.every((s) => !s)) && p.v_strings.every((row) => row.every((s) => !s)));
}
