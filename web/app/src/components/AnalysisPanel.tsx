// Controls for running engine analyses of the current position, and their results
import type { Evaluation, NimstringAnalysis } from "../engine/types";
import { formatNimstring, LOONY, signed, type AnalysisKind, type EvaluationSummary } from "../game/analysis";

// What the board shows on the legal moves
export type BoardValues = "evaluation" | "nimstring" | "off";

export interface Timed<T> {
  result: T;
  seconds: number;
}

interface AnalysisPanelProps {
  running: { kind: AnalysisKind; seconds: number } | null;
  // Running for a different position from the one shown
  runningElsewhere: boolean;
  evaluation?: Timed<Evaluation>;
  summary?: EvaluationSummary;
  nimstring?: Timed<NimstringAnalysis>;
  multiPart: boolean;
  boardValues: BoardValues;
  onBoardValues: (v: BoardValues) => void;
  // Whether evaluation values include the score so far (the final margin), or only
  // the rest of the game (the engine's own value, which ignores coins already taken)
  countScore: boolean;
  onCountScore: (count: boolean) => void;
  onRun: (kind: AnalysisKind) => void;
  onCancel: () => void;
  onHoverComponent: (index: number | null) => void;
}

const seconds = (s: number) => `${s.toFixed(1)} s`;

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
  const { running, evaluation, summary, nimstring } = props;
  const boardOptions: [BoardValues, string][] = [];
  if (evaluation) boardOptions.push(["evaluation", "Evaluation"]);
  if (nimstring) boardOptions.push(["nimstring", "Nimber"]);
  boardOptions.push(["off", "Off"]);

  return (
    <section class="analysis" aria-label="Analysis">
      <div class="analysis-actions">
        <button type="button" disabled={!!running} onClick={() => props.onRun("evaluate")}>
          Evaluate
        </button>
        <button type="button" disabled={!!running} onClick={() => props.onRun("nimstring")}>
          Nimber value
        </button>
        {running && (
          <span class="running" role="status">
            {running.kind === "evaluate" ? "Evaluating" : "Calculating Nimber value"}
            {props.runningElsewhere && " (another position)"}… {seconds(running.seconds)}
            <button type="button" class="quiet small" onClick={props.onCancel}>
              Cancel
            </button>
          </span>
        )}
      </div>

      {(evaluation || nimstring) && (
        <div class="results">
          {evaluation && summary && (
            <div class="result">
              <h2>Evaluation</h2>
              <dl>
                <dt>Value for {summary.mover}</dt>
                <dd>
                  <strong>{signed(props.countScore ? summary.margin : summary.value)}</strong>
                </dd>
                <dt>Best play ends</dt>
                <dd>
                  <strong>
                    A {summary.finalScores.A}–{summary.finalScores.B} B
                  </strong>
                </dd>
              </dl>
              <p class="timing">{seconds(evaluation.seconds)}</p>
            </div>
          )}
          {nimstring && (
            <div class="result">
              <h2>Nimstring</h2>
              <dl>
                <dt>Value</dt>
                <dd>
                  <strong>{formatNimstring(nimstring.result.value)}</strong>
                </dd>
                {nimstring.result.components.length > 1 && (
                  <>
                    <dt>Components</dt>
                    <dd>
                      <ul class="components" onMouseLeave={() => props.onHoverComponent(null)}>
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
                    </dd>
                  </>
                )}
              </dl>
              <p class="timing">{seconds(nimstring.seconds)}</p>
            </div>
          )}
        </div>
      )}

      {(evaluation || nimstring) && (
        <div class="selectors">
          <Segmented label="Values on board" options={boardOptions} value={props.boardValues} onChange={props.onBoardValues} />
          {evaluation && (
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
      )}
      {props.boardValues === "nimstring" && nimstring && <p class="hint">{LOONY} means loony.</p>}
    </section>
  );
}
