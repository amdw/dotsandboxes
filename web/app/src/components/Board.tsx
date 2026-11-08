// SVG rendering of one part of a position, in either the dots-and-boxes or the
// strings-and-coins view, with clickable lines/strings for the legal moves, and
// optionally clickable captured squares/coins (to change their owner).
import type { JSX } from "preact";
import { useState } from "preact/hooks";
import type { PartData } from "../engine/types";
import { LOONY, type MoveLabels } from "../game/analysis";
import { allLines, coinsCapturedBy, isCoinCaptured, isStringPresent, lineKey, sameLine, type Line } from "../game/lines";
import type { Player } from "../game/state";

export type View = "dab" | "snc";

export interface LastMoveBadge {
  text: string;
  kind: "best" | "inaccuracy" | "blunder";
  title: string;
}

interface BoardProps {
  data: PartData;
  part: number;
  owners: (Player | null)[][];
  view: View;
  toMove: Player;
  // The last move, if it's to be shown, and any badge judging it: its value or mark (e.g.
  // "?"), how good it was, and an explanation
  lastMove: Line | null;
  lastBadge?: LastMoveBadge | null;
  interactive: boolean;
  onPlay: (line: Line) => void;
  // If given, captured squares/coins can be clicked to change their owner
  onCycleOwner?: (x: number, y: number) => void;
  // Margins around the board (see boardMargins)
  margins: Margins;
  // CSS width of the board (its height follows from its aspect ratio)
  width: string;
  // Analysis results to show on the legal moves, keyed by lineKey
  labels?: MoveLabels;
  // Canonical moves to highlight, keyed by lineKey
  canonical?: Set<string>;
  // Coins which the highlighted canonical moves would capture
  canonicalCoins?: [number, number][];
  // voids[y][x]: cells which aren't coins, just padding (see GameState.voids)
  voids?: boolean[][] | null;
}

const CELL = 56;
const PAD = CELL * 0.65; // Margin with room for ground strings
// Margins on sides without ground strings: just enough room for the dots in the dots-and-boxes
// view; none in the strings-and-coins view, where coins are already half a square in
const DAB_SLIM_PAD = CELL * 0.25;
const GROUND = CELL * 0.4; // How far ground strings extend beyond the edge of the board
const COIN_R = CELL * 0.2;

type Point = [number, number];

// Margins around a part's board, in SVG units
export interface Margins {
  top: number;
  right: number;
  bottom: number;
  left: number;
}

// Margins for drawing a part, based on the game's starting position: a full margin only
// on sides where the strings-and-coins view has ground strings to draw. (Strings are only
// ever removed during a game, so a side never needs more room than at the start.)
export function boardMargins(start: PartData, view: View): Margins {
  if (view === "dab") return { top: DAB_SLIM_PAD, right: DAB_SLIM_PAD, bottom: DAB_SLIM_PAD, left: DAB_SLIM_PAD };
  const pad = (ground: boolean) => (ground ? PAD : 0);
  return {
    top: pad(start.h_strings[0].some((s) => s)),
    bottom: pad(start.h_strings[start.height].some((s) => s)),
    left: pad(start.v_strings.some((row) => row[0])),
    right: pad(start.v_strings.some((row) => row[start.width])),
  };
}

// Width of a part's board in SVG units, which all parts share, so boards drawn at widths
// proportional to this have the same size of squares and coins
export const naturalWidth = (data: PartData, m: Margins): number => m.left + m.right + data.width * CELL;

// Positions of the elements of a part's board
function geometry(data: PartData, m: Margins) {
  const dot = (x: number, y: number): Point => [m.left + x * CELL, m.top + y * CELL];
  const centre = (x: number, y: number): Point => [m.left + (x + 0.5) * CELL, m.top + (y + 0.5) * CELL];

  // End points of a line in the dots-and-boxes view
  const dabEnds = (line: Line): [Point, Point] => {
    const { x, y } = line;
    return line.orientation === "h" ? [dot(x, y), dot(x + 1, y)] : [dot(x, y), dot(x, y + 1)];
  };

  // End points of a string in the strings-and-coins view, and whether it goes to the ground
  const sncEnds = (line: Line): { ends: [Point, Point]; ground: boolean } => {
    const { x, y } = line;
    if (line.orientation === "h") {
      if (y === 0) return { ends: [centre(x, 0), [centre(x, 0)[0], m.top - GROUND]], ground: true };
      if (y === data.height) return { ends: [centre(x, y - 1), [centre(x, y - 1)[0], m.top + y * CELL + GROUND]], ground: true };
      return { ends: [centre(x, y - 1), centre(x, y)], ground: false };
    }
    if (x === 0) return { ends: [centre(0, y), [m.left - GROUND, centre(0, y)[1]]], ground: true };
    if (x === data.width) return { ends: [centre(x - 1, y), [m.left + x * CELL + GROUND, centre(x - 1, y)[1]]], ground: true };
    return { ends: [centre(x - 1, y), centre(x, y)], ground: false };
  };

  return { dot, centre, dabEnds, sncEnds };
}

function lineLabel(line: Line, view: View): string {
  const what = view === "dab" ? "line" : "string";
  const dir = line.orientation === "h" ? "horizontal" : "vertical";
  return `Play ${dir} ${what} at row ${line.y}, column ${line.x}`;
}

// Props making a captured square/coin a button which changes its owner
function ownerButton(x: number, y: number, owner: Player | null, view: View, onCycleOwner?: (x: number, y: number) => void) {
  if (!onCycleOwner) return {};
  const cycle = () => onCycleOwner(x, y);
  return {
    role: "button" as const,
    tabIndex: 0,
    "aria-label": `Change owner of ${view === "dab" ? "square" : "coin"} at row ${y}, column ${x} (currently ${owner ?? "nobody"})`,
    onClick: cycle,
    onKeyDown: (e: KeyboardEvent) => {
      if (e.key === "Enter" || e.key === " ") {
        e.preventDefault();
        cycle();
      }
    },
  };
}

function Segment({ ends, class: cls }: { ends: [Point, Point]; class: string }) {
  const [[x1, y1], [x2, y2]] = ends;
  return <line class={cls} x1={x1} y1={y1} x2={x2} y2={y2} />;
}

// Open arrowhead at the far end of a ground string, as in the paper's figures
function GroundArrow({ ends, class: cls }: { ends: [Point, Point]; class: string }) {
  const [[x1, y1], [x2, y2]] = ends;
  const len = Math.hypot(x2 - x1, y2 - y1);
  const [dx, dy] = [(x2 - x1) / len, (y2 - y1) / len];
  const f = CELL * 0.18; // Length of the arrowhead along the string
  const back = (side: number): string => `${x2 - f * dx - side * 0.5 * f * dy},${y2 - f * dy + side * 0.5 * f * dx}`;
  return <polyline class={cls} points={`${back(1)} ${x2},${y2} ${back(-1)}`} />;
}

export function Board({ data, part, owners, view, toMove, lastMove, lastBadge = null, interactive, onPlay, onCycleOwner, margins, width: cssWidth, labels, voids, canonical, canonicalCoins = [] }: BoardProps) {
  // Cells off the board count as voids, so that lines and dots bordering only voids are hidden
  const isVoid = (x: number, y: number) => x < 0 || y < 0 || x >= data.width || y >= data.height || !!voids?.[y][x];
  const lineBordersOnlyVoids = (line: Line) =>
    line.orientation === "h" ? isVoid(line.x, line.y - 1) && isVoid(line.x, line.y) : isVoid(line.x - 1, line.y) && isVoid(line.x, line.y);
  const { dot, centre, dabEnds, sncEnds } = geometry(data, margins);
  const width = naturalWidth(data, margins);
  const height = margins.top + margins.bottom + data.height * CELL;
  const lines = allLines(data, part);
  const coins: Point[] = [];
  for (let y = 0; y < data.height; y++) {
    for (let x = 0; x < data.width; x++) {
      coins.push([x, y]);
    }
  }

  const ends = (line: Line) => (view === "dab" ? dabEnds(line) : sncEnds(line).ends);

  // The move under the pointer (or with keyboard focus), and the coins it would capture.
  // These are previewed in separate layers (under the drawn lines in the dots-and-boxes
  // view, so the preview doesn't cover them), so this is tracked in state rather than CSS.
  const [hovered, setHovered] = useState<Line | null>(null);
  const unhover = (line: Line) => setHovered((h) => (sameLine(h, line) ? null : h));
  const previewCoins = interactive && hovered && isStringPresent(data, hovered) ? coinsCapturedBy(data, hovered) : [];
  const willCapture = (x: number, y: number) => previewCoins.some(([px, py]) => px === x && py === y);

  const moveTargets = interactive
    ? lines
        .filter((line) => isStringPresent(data, line))
        .map((line) => {
          const play = () => onPlay(line);
          const onKeyDown = (e: KeyboardEvent) => {
            if (e.key === "Enter" || e.key === " ") {
              e.preventDefault();
              play();
            }
          };
          return (
            <g
              key={`m${line.orientation}${line.x},${line.y}`}
              class="move"
              role="button"
              tabIndex={0}
              aria-label={lineLabel(line, view)}
              onClick={play}
              onKeyDown={onKeyDown}
              onPointerEnter={() => setHovered(line)}
              onPointerLeave={() => unhover(line)}
              onFocus={() => setHovered(line)}
              onBlur={() => unhover(line)}
            >
              <Segment ends={ends(line)} class="hit" />
              <Segment ends={ends(line)} class="preview" />
              {view === "snc" && sncEnds(line).ground && <GroundArrow ends={ends(line)} class="preview arrow" />}
            </g>
          );
        })
    : [];

  // Highlighted canonical moves, drawn under the move targets (and their hover previews)
  const canonicalLines = canonical
    ? lines
        .filter((line) => isStringPresent(data, line) && canonical.has(lineKey(line)))
        .map((line) => (
          <g key={`k${line.orientation}${line.x},${line.y}`}>
            <Segment ends={ends(line)} class="canonical" />
            {/* Including the arrowhead of a ground string */}
            {view === "snc" && sncEnds(line).ground && <GroundArrow ends={ends(line)} class="canonical arrow" />}
          </g>
        ))
    : [];

  // Analysis values, at the middle of each legal move's line/string
  const valueLabels = labels
    ? lines
        .filter((line) => isStringPresent(data, line) && labels.has(lineKey(line)))
        .map((line) => {
          const label = labels.get(lineKey(line))!;
          const [[x1, y1], [x2, y2]] = ends(line);
          const [cx, cy] = [(x1 + x2) / 2, (y1 + y2) / 2];
          const w = label.text.replace(/\uFE0E/g, "").length * 7.5 + 8;
          return (
            <g key={`v${line.orientation}${line.x},${line.y}`} class={label.best ? "value-label best" : "value-label"}>
              <rect x={cx - w / 2} y={cy - 9} width={w} height={18} rx={9} />
              <text x={cx} y={cy} class={label.text === LOONY ? "loony" : undefined}>
                {label.text}
              </text>
            </g>
          );
        })
    : [];

  // A badge judging the last move, just beside it (so as not to hide it): offset from the middle
  // of its line or string at right angles, towards the inside of the board
  const lastMoveMarkers: JSX.Element[] = [];
  if (lastMove && lastBadge) {
    const [[x1, y1], [x2, y2]] = ends(lastMove);
    const len = Math.hypot(x2 - x1, y2 - y1);
    const [px, py] = [-(y2 - y1) / len, (x2 - x1) / len];
    const offset = CELL * 0.26;
    let [cx, cy] = [(x1 + x2) / 2 + px * offset, (y1 + y2) / 2 + py * offset];
    if (cx < CELL * 0.3 || cy < CELL * 0.3 || cx > width - CELL * 0.3 || cy > height - CELL * 0.3) {
      [cx, cy] = [(x1 + x2) / 2 - px * offset, (y1 + y2) / 2 - py * offset];
    }
    const w = lastBadge.text.replace(/\uFE0E/g, "").length * 6.5 + 8;
    lastMoveMarkers.push(
      <g key="badge" class={`move-mark ${lastBadge.kind}`}>
        <title>{lastBadge.title}</title>
        <rect x={cx - w / 2} y={cy - 8} width={w} height={16} rx={8} />
        <text x={cx} y={cy}>
          {lastBadge.text}
        </text>
      </g>,
    );
  }

  // In the strings-and-coins view, the string the last move cut, as a thin dotted line
  // (under the coins); the coins it captured are shown as captured anyway
  const lastCut =
    view === "snc" && lastMove
      ? (() => {
          const { ends: cutEnds, ground } = sncEnds(lastMove);
          return (
            <g key="last-cut" class="last-cut">
              <Segment ends={cutEnds} class="last-cut-string" />
              {ground && <GroundArrow ends={cutEnds} class="last-cut-string arrow" />}
            </g>
          );
        })()
      : null;

  let content: (JSX.Element | null)[];
  if (view === "dab") {
    content = [
      ...coins
        .filter(([x, y]) => isCoinCaptured(data, x, y) && !isVoid(x, y))
        .map(([x, y]) => {
          const owner = owners[y][x];
          const [dx, dy] = dot(x, y);
          const [cx, cy] = centre(x, y);
          return (
            <g key={`b${x},${y}`} class={`box owner-${owner ?? "none"}${onCycleOwner ? " editable" : ""}`} {...ownerButton(x, y, owner, view, onCycleOwner)}>
              <rect x={dx} y={dy} width={CELL} height={CELL} />
              {owner && (
                <text x={cx} y={cy}>
                  {owner}
                </text>
              )}
            </g>
          );
        }),
      ...canonicalCoins.map(([x, y]) => {
        const [dx, dy] = dot(x, y);
        return <rect key={`kb${x},${y}`} class="canonical-box" x={dx} y={dy} width={CELL} height={CELL} />;
      }),
      ...previewCoins.map(([x, y]) => {
        const [dx, dy] = dot(x, y);
        return <rect key={`p${x},${y}`} class="preview-box" x={dx} y={dy} width={CELL} height={CELL} />;
      }),
      ...lines
        .filter((line) => !isStringPresent(data, line) && !lineBordersOnlyVoids(line))
        .map((line) => <Segment key={`l${line.orientation}${line.x},${line.y}`} ends={dabEnds(line)} class={sameLine(line, lastMove) ? "drawn last" : "drawn"} />),
      ...canonicalLines,
      ...moveTargets,
      ...Array.from({ length: (data.width + 1) * (data.height + 1) }, (_, i) => {
        const [dx, dy] = [i % (data.width + 1), Math.floor(i / (data.width + 1))];
        if (isVoid(dx - 1, dy - 1) && isVoid(dx, dy - 1) && isVoid(dx - 1, dy) && isVoid(dx, dy)) return null;
        const [x, y] = dot(dx, dy);
        return <circle key={`d${i}`} class="dot" cx={x} cy={y} r={CELL * 0.07} />;
      }),
    ];
  } else {
    content = [
      ...lines
        .filter((line) => isStringPresent(data, line))
        .map((line) => {
          const { ends, ground } = sncEnds(line);
          return (
            <g key={`s${line.orientation}${line.x},${line.y}`}>
              <Segment ends={ends} class="string" />
              {ground && <GroundArrow ends={ends} class="string arrow" />}
            </g>
          );
        }),
      lastCut,
      ...canonicalLines,
      ...moveTargets,
      ...coins.filter(([x, y]) => !isVoid(x, y)).map(([x, y]) => {
        const [cx, cy] = centre(x, y);
        if (!isCoinCaptured(data, x, y)) {
          const canonicalCapture = canonicalCoins.some(([kx, ky]) => kx === x && ky === y);
          const cls = willCapture(x, y) ? "coin capture-preview" : canonicalCapture ? "coin canonical-capture" : "coin";
          return <circle key={`c${x},${y}`} class={cls} cx={cx} cy={cy} r={COIN_R} />;
        }
        const owner = owners[y][x];
        return (
          <g key={`c${x},${y}`} class={`captured owner-${owner ?? "none"}${onCycleOwner ? " editable" : ""}`} {...ownerButton(x, y, owner, view, onCycleOwner)}>
            <circle cx={cx} cy={cy} r={COIN_R} />
            {owner && (
              <text x={cx} y={cy}>
                {owner}
              </text>
            )}
          </g>
        );
      }),
    ];
  }

  return (
    <svg
      class={`board view-${view} to-move-${toMove}`}
      viewBox={`0 0 ${width} ${height}`}
      style={{ width: cssWidth }}
      role="group"
      aria-label={`Part ${part}`}
    >
      {content}
      {lastMoveMarkers}
      {valueLabels}
    </svg>
  );
}
