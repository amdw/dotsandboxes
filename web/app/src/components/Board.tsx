// SVG rendering of one part of a position, in either the dots-and-boxes or the
// strings-and-coins view, with clickable lines/strings for the legal moves.
import type { JSX } from "preact";
import { useState } from "preact/hooks";
import type { PartData } from "../engine/types";
import { allLines, coinsCapturedBy, isCoinCaptured, isStringPresent, sameLine, type Line } from "../game/lines";
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

function lineLabel(line: Line, view: View): string {
  const what = view === "dab" ? "line" : "string";
  const dir = line.orientation === "h" ? "horizontal" : "vertical";
  return `Play ${dir} ${what} at row ${line.y}, column ${line.x}`;
}

function Segment({ ends, class: cls }: { ends: [Point, Point]; class: string }) {
  const [[x1, y1], [x2, y2]] = ends;
  return <line class={cls} x1={x1} y1={y1} x2={x2} y2={y2} />;
}

// Short bar across the far end of a ground string
function GroundMark({ ends, class: cls }: { ends: [Point, Point]; class: string }) {
  const [[, y1], [x2, y2]] = ends;
  const horizontal = y1 === y2;
  const h = CELL * 0.12;
  return horizontal ? (
    <line class={cls} x1={x2} y1={y2 - h} x2={x2} y2={y2 + h} />
  ) : (
    <line class={cls} x1={x2 - h} y1={y2} x2={x2 + h} y2={y2} />
  );
}

export function Board({ data, part, owners, view, toMove, lastMove, interactive, onPlay }: BoardProps) {
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
          const { ends, ground } = sncEnds(data, line);
          return (
            <g key={`s${line.orientation}${line.x},${line.y}`}>
              <Segment ends={ends} class="string" />
              {ground && <GroundMark ends={ends} class="string" />}
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
      class={`board view-${view} to-move-${toMove}`}
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
