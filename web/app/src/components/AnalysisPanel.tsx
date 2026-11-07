// Choice of engine analysis values to show on the board, with the Nimstring
// component values, and progress (with cancellation) while they're calculated
import type { NimstringAnalysis } from "../engine/types";
import { formatNimstring, LOONY, type AnalysisKind } from "../game/analysis";

// What the board shows on the legal moves
export type BoardValues = "evaluation" | "nimstring" | "off";

export interface Timed<T> {
  result: T;
  seconds: number;
}

interface AnalysisPanelProps {
  running: { kind: AnalysisKind; seconds: number } | null;
  // Nimstring results for the position shown, when Nimstring values are selected
  nimstring?: Timed<NimstringAnalysis>;
  multiPart: boolean;
  boardValues: BoardValues;
  onBoardValues: (v: BoardValues) => void;
  // Whether evaluation values include the score so far (the final margin), or only
  // the rest of the game (the engine's own value, which ignores coins already taken)
  countScore: boolean;
  onCountScore: (count: boolean) => void;
  onCancel: () => void;
  onHoverComponent: (index: number | null) => void;
}

function Segmented<T extends string | boolean>({ label, options, value, onChange }: { label: string; options: [T, string][]; value: T; onChange: (v: T) => void }) {
  return (
    <div class="selector">
      <span>{label}</span>
      <div class="segmented" role="radiogroup" aria-label={label}>
        {options.map(([v, text]) => (
          <button key={String(v)} type="button" role="radio" aria-checked={value === v} onClick={() => onChange(v)}>
            {text}
          </button>
        ))}
      </div>
    </div>
  );
}

export function AnalysisPanel(props: AnalysisPanelProps) {
  const { running, nimstring } = props;

  return (
    <section class="analysis" aria-label="Analysis">
      <div class="selectors">
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
      </div>

      {running && (
        <p class="running" role="status">
          {running.kind === "evaluate" ? "Evaluating" : "Calculating Nimber values"}… {running.seconds.toFixed(1)} s
          <button type="button" class="quiet small" onClick={props.onCancel}>
            Cancel
          </button>
        </p>
      )}

      {nimstring && (
        <div class="result">
          <h2>Nimstring value: {formatNimstring(nimstring.result.value)}</h2>
          {nimstring.result.components.length > 1 && (
            <ul class="components" aria-label="Components" onMouseLeave={() => props.onHoverComponent(null)}>
              {nimstring.result.components.map((c, i) => (
                <li
                  key={i}
                  tabIndex={0}
                  onMouseEnter={() => props.onHoverComponent(i)}
                  onFocus={() => props.onHoverComponent(i)}
                  onBlur={() => props.onHoverComponent(null)}
                >
                  {props.multiPart && `Part ${c.part}, `}
                  {c.coins.length} {c.coins.length === 1 ? "coin" : "coins"}: <strong>{formatNimstring(c.value)}</strong>
                </li>
              ))}
            </ul>
          )}
          <p class="hint">{LOONY} means loony.</p>
        </div>
      )}
    </section>
  );
}
