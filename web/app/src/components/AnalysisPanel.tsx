// Rows of the .controls grid choosing the engine analysis values to show on the board,
// and showing progress (with cancellation) while they're calculated
import type { ComponentChildren } from "preact";
import type { AnalysisKind } from "../game/analysis";

// What the board shows on the legal moves
export type BoardValues = "evaluation" | "nimstring" | "off";

export interface Timed<T> {
  result: T;
  seconds: number;
}

interface AnalysisPanelProps {
  running: { kind: AnalysisKind; seconds: number } | null;
  // Whether the analysis running is the engine thinking about its move
  engineThinking: boolean;
  boardValues: BoardValues;
  onBoardValues: (v: BoardValues) => void;
  // Whether evaluation values include the score so far (the final margin), or only
  // the rest of the game (the engine's own value, which ignores coins already taken)
  countScore: boolean;
  onCountScore: (count: boolean) => void;
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

// Progress of an analysis, with a Cancel button: shown in the .controls grid under the
// option which started it
export function AnalysisProgress({ label, seconds, onCancel }: { label: string; seconds: number; onCancel: () => void }) {
  return (
    <p class="running" role="status">
      {label}… {seconds.toFixed(1)} s
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

      {/* (When the engine is thinking about its move, the progress is shown under "Engine plays" instead) */}
      {running && !props.engineThinking && (
        <AnalysisProgress label={running.kind === "evaluate" ? "Evaluating" : "Calculating Nimber values"} seconds={running.seconds} onCancel={props.onCancel} />
      )}
    </>
  );
}
