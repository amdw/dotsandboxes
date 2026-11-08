// Rows of the .controls grid choosing the engine analysis values to show on the board,
// and showing progress (with cancellation) while they're calculated
import type { ComponentChildren } from "preact";
import type { AnalysisKind } from "../game/analysis";
import type { Player } from "../game/state";

// What the board shows on the legal moves
export type BoardValues = "evaluation" | "nimstring" | "off";

// Whose point of view evaluation values are shown from: the player to move, or always A
export type Perspective = "mover" | "A";

export interface Timed<T> {
  result: T;
  seconds: number;
}

interface AnalysisPanelProps {
  running: { kind: AnalysisKind; seconds: number; positions?: number } | null;
  // Whether the analysis running is the engine thinking about its move, or checking the
  // last move (both of which show their progress elsewhere)
  engineThinking: boolean;
  checkingMove: boolean;
  boardValues: BoardValues;
  onBoardValues: (v: BoardValues) => void;
  // Whether evaluation values include the score so far (the final margin), or only
  // the rest of the game (the engine's own value, which ignores coins already taken)
  countScore: boolean;
  onCountScore: (count: boolean) => void;
  perspective: Perspective;
  onPerspective: (perspective: Perspective) => void;
  toMove: Player;
  onCancel: () => void;
}

interface SegmentedProps<T> {
  label: string;
  options: [T, string][];
  value: T;
  onChange: (v: T) => void;
  // Shown beside the options, e.g. an action button
  children?: ComponentChildren;
}

// A labelled row of options: two cells (label and control) of the .controls grid
export function Segmented<T extends string | boolean>({ label, options, value, onChange, children }: SegmentedProps<T>) {
  return (
    <div class="selector">
      <span class="selector-label">{label}</span>
      <div class="selector-control">
        <div class="segmented" role="radiogroup" aria-label={label}>
          {options.map(([v, text]) => (
            <button key={String(v)} type="button" role="radio" aria-checked={value === v} onClick={() => onChange(v)}>
              {text}
            </button>
          ))}
        </div>
        {children}
      </div>
    </div>
  );
}

// How long an analysis has taken, and how many positions the engine has cached, e.g.
// "3.2 s · 1.4M positions"
export function progressText(seconds: number, positions?: number): string {
  const time = `${seconds.toFixed(1)} s`;
  if (!positions) return time;
  const count = positions >= 1e6 ? `${(positions / 1e6).toFixed(1)}M` : positions >= 1e3 ? `${Math.round(positions / 1e3)}k` : String(positions);
  return `${time} · ${count} positions`;
}

// Progress of an analysis, with a Cancel button: shown in the .controls grid under the
// option which started it
export function AnalysisProgress({ label, seconds, positions, onCancel }: { label: string; seconds: number; positions?: number; onCancel: () => void }) {
  return (
    <p class="running" role="status">
      {label}… {progressText(seconds, positions)}
      <button type="button" class="quiet small" onClick={onCancel}>
        Cancel
      </button>
    </p>
  );
}

export function AnalysisPanel(props: AnalysisPanelProps) {
  const { running } = props;

  return (
    <>
        <Segmented
          label="Values on board"
          options={[
            ["evaluation", "Evaluation"],
            ["nimstring", "Nimber"],
            ["off", "Off"],
          ]}
          value={props.boardValues}
          onChange={props.onBoardValues}
        />
        {props.boardValues === "evaluation" && (
          <Segmented
            label="Evaluation values"
            options={[
              [true, "Including score so far"],
              [false, "Excluding score so far"],
            ]}
            value={props.countScore}
            onChange={props.onCountScore}
          />
        )}
        {props.boardValues === "evaluation" && (
          <Segmented
            label="Values for"
            options={[
              ["A", "Player A"],
              // Naming the player to move, so it's always clear whose values these are
              ["mover", `Player to move (${props.toMove})`],
            ]}
            value={props.perspective}
            onChange={props.onPerspective}
          />
        )}

      {/* (When the engine is thinking about its move, the progress is shown under "Engine plays" instead) */}
      {running && !props.engineThinking && !props.checkingMove && (
        <AnalysisProgress label={running.kind === "evaluate" ? "Evaluating" : "Calculating Nimber values"} seconds={running.seconds} positions={running.positions} onCancel={props.onCancel} />
      )}
    </>
  );
}
