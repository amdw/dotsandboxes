// Mapping between engine moves and the lines/strings of a part, as indexed in PartData
import type { CPosMove, PartData, PositionData } from "../engine/types";

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

// Count the strings still attached to a coin
function stringCount(data: PartData, x: number, y: number): number {
  return [data.h_strings[y][x], data.h_strings[y + 1][x], data.v_strings[y][x], data.v_strings[y][x + 1]].filter((s) => s).length;
}

// The squares (coins) which drawing a line (cutting a string) would complete (capture):
// those next to the line for which it is the only string left
export function coinsCapturedBy(data: PartData, line: Line): [number, number][] {
  const { x, y } = line;
  const adjacent: [number, number][] =
    line.orientation === "h"
      ? [[x, y - 1], [x, y]].filter(([, cy]) => cy >= 0 && cy < data.height) as [number, number][]
      : [[x - 1, y], [x, y]].filter(([cx]) => cx >= 0 && cx < data.width) as [number, number][];
  return isStringPresent(data, line) ? adjacent.filter(([cx, cy]) => stringCount(data, cx, cy) === 1) : [];
}

// Add or remove a single string, for setting up positions (not a move: no captures are scored)
export function toggleString(position: PositionData, line: Line): PositionData {
  const parts = position.parts.map((p, i) => {
    if (i !== line.part) return p;
    const flip = (rows: boolean[][], orientation: Line["orientation"]) =>
      line.orientation === orientation ? rows.map((row, y) => (y === line.y ? row.map((s, x) => (x === line.x ? !s : s)) : row)) : rows;
    return { ...p, h_strings: flip(p.h_strings, "h"), v_strings: flip(p.v_strings, "v") };
  });
  return { parts };
}
