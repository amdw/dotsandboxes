// Encoding a game in the fragment of a URL (the part after "#"), so that it can be shared as
// a link, or restored on reload. Fields, in URLSearchParams form:
//   p  the starting position: parts separated by ",", each "WxH." followed by a bitmap of the
//      strings present (horizontal lines row by row, then vertical lines), in base64url
//   m  the moves played, each the index of its line among all the position's lines (parts in
//      order, each listed as above), as a fixed number of base64url digits
//   c  the move shown, if not the last
//   t  "B" if B moves first
//   o  if any coins were captured (with owners) before the start, or there are voids: one
//      character per cell (parts, rows, columns in order): "A" or "B" for an owner,
//      "v" for a void, "." otherwise
//   s  scores from before the start other than for the owned coins, as "A,B", if not 0,0
//   v  "snc" for the strings-and-coins view
import type { PartData, PositionData } from "../engine/types";
import type { View } from "../components/Board";
import { allLines, lineToMove, moveToLine, type Line } from "./lines";
import { current, goTo, newGameState, playMove, startFrom, type GameState, type MakeMove, type Player } from "./state";

const DIGITS = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
// Limits on what a link may describe, matching the engine's own
const MAX_DIMENSION = 20;
const MAX_PARTS = 16;

function digit(c: string): number {
  const d = DIGITS.indexOf(c);
  if (d < 0 || c.length !== 1) throw new Error(`Invalid character "${c}"`);
  return d;
}

function encodeBits(bits: boolean[]): string {
  let out = "";
  for (let i = 0; i < bits.length; i += 6) {
    let d = 0;
    for (let j = 0; j < 6; j++) d = (d << 1) | (bits[i + j] ? 1 : 0);
    out += DIGITS[d];
  }
  return out;
}

function decodeBits(text: string, count: number): boolean[] {
  if (text.length !== Math.ceil(count / 6)) throw new Error("Wrong number of strings in position");
  const bits: boolean[] = [];
  for (const c of text) {
    const d = digit(c);
    for (let j = 5; j >= 0; j--) bits.push(((d >> j) & 1) === 1);
  }
  return bits.slice(0, count);
}

function partStrings(part: PartData): boolean[] {
  return [...part.h_strings.flat(), ...part.v_strings.flat()];
}

// Number of base64url digits used for each move, for a position with this many lines
function moveWidth(lineCount: number): number {
  let width = 1;
  while (64 ** width < lineCount) width++;
  return width;
}

function lineIndex(position: PositionData, line: Line): number {
  let offset = 0;
  for (let i = 0; i < line.part; i++) offset += allLines(position.parts[i], i).length;
  const { width, height } = position.parts[line.part];
  return offset + (line.orientation === "h" ? line.y * width + line.x : (height + 1) * width + line.y * (width + 1) + line.x);
}

export function encodeGame(game: GameState, view: View): string {
  const start = game.snapshots[0];
  const position = start.position;
  const params = new URLSearchParams();
  params.set("p", position.parts.map((p) => `${p.width}x${p.height}.${encodeBits(partStrings(p))}`).join(","));

  const moves = game.snapshots.slice(1).map((s) => s.lastMove!);
  if (moves.length > 0) {
    const lineCount = position.parts.reduce((n, p, i) => n + allLines(p, i).length, 0);
    const width = moveWidth(lineCount);
    params.set(
      "m",
      moves
        .map((m) => {
          let index = lineIndex(position, moveToLine(m));
          let out = "";
          for (let i = 0; i < width; i++) {
            out = DIGITS[index % 64] + out;
            index = Math.floor(index / 64);
          }
          return out;
        })
        .join(""),
    );
    if (game.cursor < moves.length) params.set("c", String(game.cursor));
  }
  if (start.toMove === "B") params.set("t", "B");

  const cells = position.parts.flatMap((p, i) =>
    Array.from({ length: p.height }, (_, y) =>
      Array.from({ length: p.width }, (_, x) => (game.voids?.[i][y][x] ? "v" : (start.owners[i][y][x] ?? "."))),
    ).flat(),
  );
  if (cells.some((c) => c !== ".")) params.set("o", cells.join(""));
  if (game.baseScores.A !== 0 || game.baseScores.B !== 0) params.set("s", `${game.baseScores.A},${game.baseScores.B}`);
  if (view === "snc") params.set("v", "snc");
  // Every value only uses characters which are fine in a URL fragment (letters, digits and
  // "-_.,"), so join them directly rather than with URLSearchParams.toString, which would
  // needlessly escape the commas
  return [...params].map(([key, value]) => `${key}=${value}`).join("&");
}

export interface SharedGame {
  game: GameState;
  view: View | null;
}

// Decode a game from a URL fragment (with or without the "#"), playing its moves with
// makeMove (which throws on illegal moves). Throws an Error describing any problem.
export function decodeGame(fragment: string, makeMove: MakeMove): SharedGame {
  const params = new URLSearchParams(fragment.replace(/^#/, ""));
  const p = params.get("p");
  if (!p) throw new Error("No position in link");

  const parts: PartData[] = p.split(",").map((spec) => {
    const match = /^(\d+)x(\d+)\.([A-Za-z0-9_-]*)$/.exec(spec);
    if (!match) throw new Error(`Invalid part "${spec}"`);
    const [width, height] = [Number(match[1]), Number(match[2])];
    if (width < 1 || height < 1 || width > MAX_DIMENSION || height > MAX_DIMENSION) throw new Error(`Invalid part size ${width}x${height}`);
    const bits = decodeBits(match[3], (height + 1) * width + height * (width + 1));
    const h = (height + 1) * width;
    return {
      width,
      height,
      h_strings: Array.from({ length: height + 1 }, (_, y) => bits.slice(y * width, (y + 1) * width)),
      v_strings: Array.from({ length: height }, (_, y) => bits.slice(h + y * (width + 1), h + (y + 1) * (width + 1))),
    };
  });
  if (parts.length > MAX_PARTS) throw new Error(`Too many parts (${parts.length})`);
  const position: PositionData = { parts };

  const cellCount = parts.reduce((n, part) => n + part.width * part.height, 0);
  const o = params.get("o");
  if (o !== null && (o.length !== cellCount || !/^[.ABv]*$/.test(o))) throw new Error("Invalid owners in link");
  let cell = 0;
  const owners: (Player | null)[][][] = [];
  const voids: boolean[][][] = [];
  for (const part of parts) {
    const partOwners: (Player | null)[][] = [];
    const partVoids: boolean[][] = [];
    for (let y = 0; y < part.height; y++) {
      const ownerRow: (Player | null)[] = [];
      const voidRow: boolean[] = [];
      for (let x = 0; x < part.width; x++) {
        const c = o?.[cell++] ?? ".";
        ownerRow.push(c === "A" || c === "B" ? c : null);
        voidRow.push(c === "v");
      }
      partOwners.push(ownerRow);
      partVoids.push(voidRow);
    }
    owners.push(partOwners);
    voids.push(partVoids);
  }

  const s = params.get("s");
  let baseScores = { A: 0, B: 0 };
  if (s !== null) {
    const match = /^(\d+),(\d+)$/.exec(s);
    if (!match) throw new Error("Invalid scores in link");
    baseScores = { A: Number(match[1]), B: Number(match[2]) };
  }

  const t = params.get("t");
  if (t !== null && t !== "B") throw new Error("Invalid first player in link");
  const start = current(newGameState(position));
  let game = startFrom(
    { ...start, owners, toMove: t === "B" ? "B" : "A" },
    { baseScores, voids: voids.some((part) => part.some((row) => row.some((v) => v))) ? voids : null },
  );

  const m = params.get("m");
  if (m) {
    const lines = parts.flatMap((part, i) => allLines(part, i));
    const width = moveWidth(lines.length);
    if (m.length % width !== 0) throw new Error("Invalid moves in link");
    for (let i = 0; i < m.length; i += width) {
      let index = 0;
      for (const c of m.slice(i, i + width)) index = index * 64 + digit(c);
      if (index >= lines.length) throw new Error("Invalid move in link");
      if (current(game).gameOver) throw new Error("Moves after the end of the game in link");
      game = playMove(game, lineToMove(lines[index]), makeMove);
    }
  }

  const c = params.get("c");
  if (c !== null) {
    if (!/^\d+$/.test(c) || Number(c) > game.snapshots.length - 1) throw new Error("Invalid move number in link");
    game = goTo(game, Number(c));
  }

  const v = params.get("v");
  return { game, view: v === "snc" ? "snc" : v === null ? null : "dab" };
}
