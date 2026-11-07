// Controls for running engine analyses of the current position, and their results
import type { Evaluation, NimstringAnalysis } from "../engine/types";
import { formatNimstring, LOONY, signed, type AnalysisKind, type EvaluationSummary } from "../game/analysis";

// What the board shows on the legal moves
export type BoardValues = "engine" | "margin" | "nimstring" | "off";

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
  onRun: (kind: AnalysisKind) => void;
  onCancel: () => void;
  onHoverComponent: (index: number | null) => void;
}

const seconds = (s: number) => `${s.toFixed(1)} s`;

export function AnalysisPanel(props: AnalysisPanelProps) {
  const { running, evaluation, summary, nimstring } = props;
  const boardOptions: [BoardValues, string, boolean][] = [
    ["engine", "Engine value", !!evaluation],
    ["margin", "Final margin", !!evaluation],
    ["nimstring", "Nimber", !!nimstring],
    ["off", "Off", true],
  ];

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
                <dt>Engine value</dt>
                <dd>
                  <strong>{signed(summary.value)}</strong> for {summary.mover}, from the rest of the game
                </dd>
                <dt>Final margin</dt>
                <dd>
                  <strong>{signed(summary.margin)}</strong> for {summary.mover}, counting coins already taken: best play ends{" "}
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
        <div class="board-values">
          <span id="board-values-label">Values on board</span>
          <div class="segmented" role="radiogroup" aria-labelledby="board-values-label">
            {boardOptions
              .filter(([, , available]) => available)
              .map(([value, label]) => (
                <button key={value} type="button" role="radio" aria-checked={props.boardValues === value} onClick={() => props.onBoardValues(value)}>
                  {label}
                </button>
              ))}
          </div>
        </div>
      )}
      {summary && (props.boardValues === "engine" || props.boardValues === "margin") && (
        <p class="hint">
          {props.boardValues === "engine"
            ? `Each move's value is the net coins ${summary.mover} gains from the rest of the game by making it.`
            : `Each move's value is ${summary.mover}'s final winning margin (negative if losing) after making it, counting the ${signed(summary.scoreDiff)} lead so far.`}{" "}
          Best moves are highlighted.
        </p>
      )}
      {props.boardValues === "nimstring" && nimstring && <p class="hint">Each move's value is the Nimstring value of the position after it ({LOONY} means loony).</p>}
    </section>
  );
}
