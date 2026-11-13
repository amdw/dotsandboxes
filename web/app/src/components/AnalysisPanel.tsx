// Rows of the .controls grid choosing the engine analysis values to show on the board,
// and showing progress (with cancellation) while they're calculated
import type { ComponentChildren } from "preact";
import type { AnalysisKind } from "../game/analysis";
import type { Progress } from "../engine/analyser";

// What the board shows on the legal moves
export type BoardValues = "evaluation" | "nimstring" | "off";

// Whose point of view evaluation values are shown from: the player to move, or always A.
// (These, and whether evaluation values include the score so far (the final margin) or only
// the rest of the game (the engine's own value, which ignores coins already taken), are
// chosen in the settings: see MoreSettings.)
export type Perspective = "mover" | "A";

export interface Timed<T> {
  result: T;
  seconds: number;
}

interface AnalysisPanelProps {
  running: { kind: AnalysisKind; seconds: number; progress?: Progress } | null;
  // Whether the analysis running is the engine thinking about its move, or checking the
  // last move (both of which show their progress elsewhere)
  engineThinking: boolean;
  checkingMove: boolean;
  boardValues: BoardValues;
  onBoardValues: (v: BoardValues) => void;
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

// A count of positions, e.g. "1.4M"
function count(n: number): string {
  return n >= 1e6 ? `${(n / 1e6).toFixed(1)}M` : n >= 1e3 ? `${Math.round(n / 1e3)}k` : String(n);
}

// How long an analysis has taken, e.g. "3.2 s"
export function elapsedText(seconds: number): string {
  return `${seconds.toFixed(1)} s`;
}

// How many positions an analysis has calculated, and how full the engine's cache is, e.g.
// "1.4M positions · cache 12% full", or once the analysis has had to forget values to make
// room, how many, e.g. "21.0M positions · cache full, 7.3M forgotten". Null until there is
// any progress to show.
export function progressText(progress?: Progress): string | null {
  if (!progress?.positions) return null;
  const cache =
    progress.forgotten > 0 ? `cache full, ${count(progress.forgotten)} forgotten` : `cache ${Math.floor((100 * progress.cached) / progress.capacity)}% full`;
  return `${count(progress.positions)} positions · ${cache}`;
}

// Progress of an analysis: the time so far, with a Cancel button, then (more quietly) the
// positions calculated and the state of the cache. Shown in the .controls grid under the
// option which started it.
export function AnalysisProgress({ label, seconds, progress, onCancel }: { label: string; seconds: number; progress?: Progress; onCancel: () => void }) {
  const detail = progressText(progress);
  return (
    <div class="running" role="status">
      <div>
        {label}… {elapsedText(seconds)}
        <button type="button" class="quiet small" onClick={onCancel}>
          Cancel
        </button>
      </div>
      {detail && <div class="progress-detail">{detail}</div>}
    </div>
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

      {/* (When the engine is thinking about its move, the progress is shown under "Engine plays" instead) */}
      {running && !props.engineThinking && !props.checkingMove && (
        <AnalysisProgress label={running.kind === "evaluate" ? "Evaluating" : "Calculating Nimber values"} seconds={running.seconds} progress={running.progress} onCancel={props.onCancel} />
      )}
    </>
  );
}
