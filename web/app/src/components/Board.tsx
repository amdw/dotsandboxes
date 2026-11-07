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

interface BoardProps {
  data: PartData;
  part: number;
  owners: (Player | null)[][];
  view: View;
  toMove: Player;
  lastMove: Line | null;
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

export function Board({ data, part, owners, view, toMove, lastMove, interactive, onPlay, onCycleOwner, margins, width: cssWidth, labels }: BoardProps) {
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

  let content: JSX.Element[];
  if (view === "dab") {
    content = [
      ...coins
        .filter(([x, y]) => isCoinCaptured(data, x, y))
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
      ...previewCoins.map(([x, y]) => {
        const [dx, dy] = dot(x, y);
        return <rect key={`p${x},${y}`} class="preview-box" x={dx} y={dy} width={CELL} height={CELL} />;
      }),
      ...lines
        .filter((line) => !isStringPresent(data, line))
        .map((line) => <Segment key={`l${line.orientation}${line.x},${line.y}`} ends={dabEnds(line)} class={sameLine(line, lastMove) ? "drawn last" : "drawn"} />),
      ...moveTargets,
      ...Array.from({ length: (data.width + 1) * (data.height + 1) }, (_, i) => {
        const [x, y] = dot(i % (data.width + 1), Math.floor(i / (data.width + 1)));
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
      ...moveTargets,
      ...coins.map(([x, y]) => {
        const [cx, cy] = centre(x, y);
        if (!isCoinCaptured(data, x, y)) {
          return <circle key={`c${x},${y}`} class={willCapture(x, y) ? "coin capture-preview" : "coin"} cx={cx} cy={cy} r={COIN_R} />;
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
      {valueLabels}
    </svg>
  );
}
