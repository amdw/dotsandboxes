// SVG rendering of one part of a position, in either the dots-and-boxes or the
// strings-and-coins view, with clickable lines/strings for the legal moves, or in
// edit mode, for adding and removing any line/string to set up a position.
import type { JSX } from "preact";
import { useState } from "preact/hooks";
import type { PartData } from "../engine/types";
import { allLines, coinsCapturedBy, isCoinCaptured, isStringPresent, sameLine, type Line } from "../game/lines";
import type { Player } from "../game/state";

export type View = "dab" | "snc";
export type Mode = "play" | "edit";

interface BoardProps {
  data: PartData;
  part: number;
  owners: (Player | null)[][];
  view: View;
  mode: Mode;
  toMove: Player;
  lastMove: Line | null;
  interactive: boolean;
  onPlay: (line: Line) => void;
}

const CELL = 56;
const PAD = CELL * 0.65;
const GROUND = CELL * 0.4; // How far ground strings extend beyond the edge of the board
const COIN_R = CELL * 0.2;

type Point = [number, number];

const dot = (x: number, y: number): Point => [PAD + x * CELL, PAD + y * CELL];
const centre = (x: number, y: number): Point => [PAD + (x + 0.5) * CELL, PAD + (y + 0.5) * CELL];

// End points of a line in the dots-and-boxes view
function dabEnds(line: Line): [Point, Point] {
  const { x, y } = line;
  return line.orientation === "h" ? [dot(x, y), dot(x + 1, y)] : [dot(x, y), dot(x, y + 1)];
}

// End points of a string in the strings-and-coins view, and whether it goes to the ground
function sncEnds(data: PartData, line: Line): { ends: [Point, Point]; ground: boolean } {
  const { x, y } = line;
  if (line.orientation === "h") {
    if (y === 0) return { ends: [centre(x, 0), [centre(x, 0)[0], PAD - GROUND]], ground: true };
    if (y === data.height) return { ends: [centre(x, y - 1), [centre(x, y - 1)[0], PAD + y * CELL + GROUND]], ground: true };
    return { ends: [centre(x, y - 1), centre(x, y)], ground: false };
  }
  if (x === 0) return { ends: [centre(0, y), [PAD - GROUND, centre(0, y)[1]]], ground: true };
  if (x === data.width) return { ends: [centre(x - 1, y), [PAD + x * CELL + GROUND, centre(x - 1, y)[1]]], ground: true };
  return { ends: [centre(x - 1, y), centre(x, y)], ground: false };
}

function lineLabel(line: Line, view: View, mode: Mode, present: boolean): string {
  const what = view === "dab" ? "line" : "string";
  const dir = line.orientation === "h" ? "horizontal" : "vertical";
  // A present string is an undrawn line, and vice versa
  const verb = mode === "play" ? "Play" : view === "dab" ? (present ? "Draw" : "Erase") : present ? "Cut" : "Restore";
  return `${verb} ${dir} ${what} at row ${line.y}, column ${line.x}`;
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

export function Board({ data, part, owners, view, mode, toMove, lastMove, interactive, onPlay }: BoardProps) {
  const width = 2 * PAD + data.width * CELL;
  const height = 2 * PAD + data.height * CELL;
  const lines = allLines(data, part);
  const coins: Point[] = [];
  for (let y = 0; y < data.height; y++) {
    for (let x = 0; x < data.width; x++) {
      coins.push([x, y]);
    }
  }

  const ends = (line: Line) => (view === "dab" ? dabEnds(line) : sncEnds(data, line).ends);

  // The move under the pointer (or with keyboard focus), and the coins it would capture.
  // These are previewed in separate layers (under the drawn lines in the dots-and-boxes
  // view, so the preview doesn't cover them), so this is tracked in state rather than CSS.
  const [hovered, setHovered] = useState<Line | null>(null);
  const unhover = (line: Line) => setHovered((h) => (sameLine(h, line) ? null : h));
  const previewCoins = interactive && mode === "play" && hovered && isStringPresent(data, hovered) ? coinsCapturedBy(data, hovered) : [];
  const willCapture = (x: number, y: number) => previewCoins.some(([px, py]) => px === x && py === y);

  const moveTargets = interactive
    ? lines
        .filter((line) => mode === "edit" || isStringPresent(data, line))
        .map((line) => {
          const present = isStringPresent(data, line);
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
              class={mode === "play" ? "move" : present ? "move add" : "move remove"}
              role="button"
              tabIndex={0}
              aria-label={lineLabel(line, view, mode, present)}
              onClick={play}
              onKeyDown={onKeyDown}
              onPointerEnter={() => setHovered(line)}
              onPointerLeave={() => unhover(line)}
              onFocus={() => setHovered(line)}
              onBlur={() => unhover(line)}
            >
              <Segment ends={ends(line)} class="hit" />
              <Segment ends={ends(line)} class="preview" />
              {view === "snc" && sncEnds(data, line).ground && <GroundArrow ends={ends(line)} class="preview arrow" />}
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
            <g key={`b${x},${y}`} class={`box owner-${owner ?? "none"}`}>
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
      ...(mode === "edit"
        ? lines.filter((line) => isStringPresent(data, line)).map((line) => <Segment key={`g${line.orientation}${line.x},${line.y}`} ends={dabEnds(line)} class="ghost" />)
        : []),
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
      ...(mode === "edit"
        ? lines
            .filter((line) => !isStringPresent(data, line))
            .map((line) => {
              const { ends, ground } = sncEnds(data, line);
              return (
                <g key={`g${line.orientation}${line.x},${line.y}`}>
                  <Segment ends={ends} class="ghost" />
                  {ground && <GroundArrow ends={ends} class="ghost arrow" />}
                </g>
              );
            })
        : []),
      ...lines
        .filter((line) => isStringPresent(data, line))
        .map((line) => {
          const { ends, ground } = sncEnds(data, line);
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
          <g key={`c${x},${y}`} class={`captured owner-${owner ?? "none"}`}>
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
      class={`board view-${view} ${mode === "play" ? `to-move-${toMove}` : "mode-edit"}`}
      viewBox={`0 0 ${width} ${height}`}
      // Allow some enlargement, since the natural size is small for small boards
      style={{ maxWidth: `${Math.round(width * 1.4)}px` }}
      role="group"
      aria-label={`Part ${part}`}
    >
      {content}
    </svg>
  );
}
