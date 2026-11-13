import { useCallback, useEffect, useLayoutEffect, useRef, useState } from "preact/hooks";
import { addProgress, Analyser, CancelledError, TimeLimitError, type Progress } from "./engine/analyser";
import { engine, errorMessage, loadEngine } from "./engine/engine";
import type { CPosMove, Evaluation, NimstringAnalysis, NimstringValue, PositionData } from "./engine/types";
import { AnalysisPanel, AnalysisProgress, elapsedText, progressText, Segmented, type BoardValues, type Perspective, type Timed } from "./components/AnalysisPanel";
import { Board, boardMargins, naturalWidth, type LastMoveBadge, type View } from "./components/Board";
import { ConfirmPopover } from "./components/ConfirmPopover";
import { DEFAULT_ENGINE_TIME_LIMIT, formatTimeLimit, isEngineTimeLimit, MoreSettings, timeLimitSeconds, type EngineTimeLimit } from "./components/SettingsPanel";
import { ExportPanel, ImportPanel, NewGamePanel } from "./components/SetupPanels";
import { chooseEngineMove, evaluationLabels, judgeMove, formatNimstring, lineKeyOf, LOONY, nimstringLabels, partValues, positionKey, type AnalysisKind, type MoveLabels } from "./game/analysis";
import { isCoinCaptured, lineKey, lineToMove, moveToLine, type Line } from "./game/lines";
import { decodeGame, encodeGame } from "./game/share";
import { canonicalSequence, current, goTo, isolatePart, newGameState, nextOwner, other, playCanonicalMoves, playMove, restart, startFromCurrent, startSplit, withStart, type GameState, type Player } from "./game/state";

const VIEW_KEY = "dab-view";
// Pause before the engine plays a move it has chosen, so that playouts can be followed
const ENGINE_DELAY_MS = 600;
// Most positions not already cached by the engine which a "cheap" analysis may calculate
// (a fraction of a second's work), when deciding whether to keep showing values on the
// board after moving to another position
const CHEAP_ANALYSIS_POSITIONS = 100_000;
const COUNT_SCORE_KEY = "dab-count-score";
const SHOW_CANONICAL_KEY = "dab-show-canonical";
const LAST_MOVE_KEY = "dab-last-move";
// The earlier, on/off form of the setting
const SHOW_LAST_MOVE_KEY = "dab-show-last-move";
const ENGINE_TIME_LIMIT_KEY = "dab-engine-time-limit";

// Whether to show the last move, and if so whether to judge it ("?" or "??" for a mistake)
type LastMoveDisplay = "hide" | "show" | "judge";
// Boards may be enlarged up to this much, since their natural size is small for small boards
const MAX_ENLARGEMENT = 1.4;

type Panel = "new" | "import" | "export";

function loadView(): View {
  try {
    return localStorage.getItem(VIEW_KEY) === "snc" ? "snc" : "dab";
  } catch {
    return "dab";
  }
}

// Whether evaluation values include the score so far (the default), as remembered
function loadCountScore(): boolean {
  try {
    return localStorage.getItem(COUNT_SCORE_KEY) !== "false";
  } catch {
    return true;
  }
}

// Whose point of view evaluation values are shown from, remembered separately for values
// including and excluding the score so far. By default, values including the score are for
// A (so their sign doesn't flip as the turn changes), and those excluding it for the
// player to move.
function perspectiveKey(countScore: boolean): string {
  return countScore ? "dab-perspective-with-score" : "dab-perspective-without-score";
}

function loadPerspectives(): Record<"withScore" | "withoutScore", Perspective> {
  const load = (countScore: boolean, fallback: Perspective): Perspective => {
    try {
      const stored = localStorage.getItem(perspectiveKey(countScore));
      return stored === "A" || stored === "mover" ? stored : fallback;
    } catch {
      return fallback;
    }
  };
  return { withScore: load(true, "A"), withoutScore: load(false, "mover") };
}

function savePerspective(countScore: boolean, perspective: Perspective) {
  try {
    localStorage.setItem(perspectiveKey(countScore), perspective);
  } catch {
    // Not important if the preference can't be saved
  }
}

function saveCountScore(count: boolean) {
  try {
    localStorage.setItem(COUNT_SCORE_KEY, String(count));
  } catch {
    // Not important if the preference can't be saved
  }
}

function loadLastMoveDisplay(): LastMoveDisplay {
  try {
    const stored = localStorage.getItem(LAST_MOVE_KEY);
    if (stored === "hide" || stored === "show" || stored === "judge") return stored;
    return localStorage.getItem(SHOW_LAST_MOVE_KEY) === "false" ? "hide" : "judge";
  } catch {
    return "judge";
  }
}

function saveLastMoveDisplay(display: LastMoveDisplay) {
  try {
    localStorage.setItem(LAST_MOVE_KEY, display);
  } catch {
    // Not important if the preference can't be saved
  }
}

// How a final margin for a player reads, e.g. "wins by 3"
function outcome(margin: number): string {
  return margin > 0 ? `wins by ${margin}` : margin < 0 ? `loses by ${-margin}` : "draws";
}

function loadShowCanonical(): boolean {
  try {
    return localStorage.getItem(SHOW_CANONICAL_KEY) === "true";
  } catch {
    return false;
  }
}

function saveShowCanonical(show: boolean) {
  try {
    localStorage.setItem(SHOW_CANONICAL_KEY, String(show));
  } catch {
    // Not important if the preference can't be saved
  }
}

function loadEngineTimeLimit(): EngineTimeLimit {
  try {
    const stored = localStorage.getItem(ENGINE_TIME_LIMIT_KEY);
    return isEngineTimeLimit(stored) ? stored : DEFAULT_ENGINE_TIME_LIMIT;
  } catch {
    return DEFAULT_ENGINE_TIME_LIMIT;
  }
}

function saveEngineTimeLimit(limit: EngineTimeLimit) {
  try {
    localStorage.setItem(ENGINE_TIME_LIMIT_KEY, limit);
  } catch {
    // Not important if the preference can't be saved
  }
}

function saveView(view: View) {
  try {
    localStorage.setItem(VIEW_KEY, view);
  } catch {
    // Not important if the preference can't be saved
  }
}

// A game from a starting position, with any given moves already played but
// the cursor at the start, so they can be stepped through
function gameFrom(position: PositionData, moves: CPosMove[] = []): GameState {
  let game = newGameState(position);
  for (const m of moves) {
    game = playMove(game, m, engine.makeMove);
  }
  return goTo(game, 0);
}

function sizesOf(position: PositionData): [number, number][] {
  return position.parts.map((p) => [p.width, p.height]);
}

function NimstringValueText({ value }: { value: NimstringValue }) {
  const text = formatNimstring(value);
  return <span class={text === LOONY ? "loony" : undefined}>{text}</span>;
}

// The engine having run out of time thinking about a position (see App)
interface TimedOut {
  key: string;
  // Its time limit
  seconds: number;
  // How long it had been thinking about the position, and its progress, in all
  elapsedMs: number;
  progress?: Progress;
}

export function App() {
  const [game, setGame] = useState<GameState | null>(null);
  const [name, setName] = useState("position");
  const [loadError, setLoadError] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [view, setView] = useState<View>(loadView);
  const [panel, setPanel] = useState<Panel | null>(null);
  // Which players the engine plays, and whether it is paused (e.g. after moving back
  // through the game, so as not to overwrite the moves after the position shown)
  const [engineSides, setEngineSides] = useState<Record<Player, boolean>>({ A: false, B: false });
  const [paused, setPaused] = useState(false);
  // The position (by positionKey) the engine last ran out of time on, its time limit then, and
  // how long it had been thinking about it (in ms) and its progress: while it's paused there, a
  // message says so, with a button to resume
  const [timedOut, setTimedOut] = useState<TimedOut | null>(null);
  // When the engine resumes after running out of time, so its next analysis of that position
  // shows the time and progress so far, carrying on from them
  const resumingFrom = useRef<TimedOut | null>(null);
  // How long the engine may think about a move before giving up (and pausing)
  const [engineTimeLimit, setEngineTimeLimit] = useState(loadEngineTimeLimit);
  // Whether to highlight canonical moves (captures which can be made without loss)
  const [showCanonical, setShowCanonical] = useState(loadShowCanonical);
  // Whether to show the last move made, and marks judging it
  const [lastMoveDisplay, setLastMoveDisplay] = useState(loadLastMoveDisplay);
  const showLastMove = lastMoveDisplay !== "hide";
  // Judging the last move needs the position before it evaluated, so it's only done if wanted
  const judgeLastMove = lastMoveDisplay === "judge";
  // The part whose "play on its own" confirmation is open, if any
  const [isolating, setIsolating] = useState<number | null>(null);
  const closeIsolating = useCallback(() => setIsolating(null), []);

  // Engine analysis: results are cached per position (keyed by positionKey)
  const analyser = useRef<Analyser | null>(null);
  const [results, setResults] = useState<Map<string, { evaluate?: Timed<Evaluation>; nimstring?: Timed<NimstringAnalysis> }>>(new Map());
  // The full analysis running, if any, and its progress
  const [running, setRunning] = useState<{ kind: AnalysisKind; key: string; start: number; progress?: Progress } | null>(null);
  const [now, setNow] = useState(0);
  // Values are only calculated when chosen, since analysing larger positions takes a long
  // time, so this is off unless the chosen values are available for the position shown
  const [boardValues, setBoardValues] = useState<BoardValues>("off");
  const [countScore, setCountScore] = useState(loadCountScore);
  const [perspectives, setPerspectives] = useState(loadPerspectives);
  const perspective = countScore ? perspectives.withScore : perspectives.withoutScore;

  // Tick while an analysis runs, to show the time taken so far
  useEffect(() => {
    if (!running) return;
    const timer = setInterval(() => setNow(performance.now()), 100);
    return () => clearInterval(timer);
  }, [running]);

  useEffect(() => () => analyser.current?.cancel(), []);

  // The position shown and the values wanted on the board, also as refs for use when an
  // analysis completes (which may be after they've changed)
  const shownSnapshot = game ? current(game) : null;
  const shownPosition = shownSnapshot?.position ?? null;
  const shownKey = shownPosition ? positionKey(shownPosition) : null;
  const wanted: AnalysisKind | null = boardValues === "evaluation" ? "evaluate" : boardValues === "nimstring" ? "nimstring" : null;
  // Whether the engine is about to play (it evaluates the position first)
  const engineToMove = !!shownSnapshot && !shownSnapshot.gameOver && engineSides[shownSnapshot.toMove];
  const engineActive = engineToMove && !paused;
  const shownKeyRef = useRef(shownKey);
  shownKeyRef.current = shownKey;
  const wantedRef = useRef(wanted);
  wantedRef.current = wanted;
  // Cheap analyses in progress (as "positionKey kind"), so each is only requested once, and
  // those which gave up because the analysis wasn't cheap
  const quickPending = useRef(new Set<string>());
  const [notCheap, setNotCheap] = useState<Set<string>>(new Set());

  const storeResult = (key: string, kind: AnalysisKind, result: Evaluation | NimstringAnalysis, start: number) => {
    const timed = { result, seconds: (performance.now() - start) / 1000 };
    setResults((prev) => new Map(prev).set(key, { ...prev.get(key), [kind]: timed }));
  };

  // Try calculating values for the board only if that's cheap (e.g. because the engine
  // already explored the position when analysing an earlier one), switching the board
  // values off if not, so the user can choose whether to start a longer calculation
  const analyseIfCheap = (kind: AnalysisKind, position: PositionData, key: string) => {
    const id = `${key} ${kind}`;
    if (quickPending.current.has(id)) return;
    quickPending.current.add(id);
    const a = (analyser.current ??= new Analyser());
    const start = performance.now();
    const stillWanted = () => shownKeyRef.current === key && wantedRef.current === kind;
    a.run(kind, position, { maxNew: CHEAP_ANALYSIS_POSITIONS })
      .then((result) => {
        if (result) storeResult(key, kind, result, start);
        else {
          setNotCheap((prev) => new Set(prev).add(id));
          if (stillWanted()) setBoardValues("off");
        }
      })
      .catch(() => {
        if (stillWanted()) setBoardValues("off");
      })
      .finally(() => quickPending.current.delete(id));
  };

  // When the position shown changes, stop any long analysis of the previous one, and show
  // the values wanted on the board if they're available or cheap to calculate
  useEffect(() => {
    if (running && running.key !== shownKey) {
      analyser.current?.cancel();
      setRunning(null);
    }
    const coming = (running?.key === shownKey && running.kind === wanted) || (wanted === "evaluate" && engineActive);
    if (wanted && shownPosition && shownKey && !results.get(shownKey)?.[wanted] && !coming) {
      analyseIfCheap(wanted, shownPosition, shownKey);
    }
  }, [shownKey]);

  // To judge the last move, evaluate the position before it, but only if that's cheap (e.g.
  // the engine explored it while evaluating or playing), never starting a long calculation
  const previousSnapshot = game && game.cursor > 0 ? game.snapshots[game.cursor - 1] : null;
  const previousKey = previousSnapshot ? positionKey(previousSnapshot.position) : null;
  useEffect(() => {
    if (judgeLastMove && previousSnapshot && previousKey && !results.get(previousKey)?.evaluate) {
      analyseIfCheap("evaluate", previousSnapshot.position, previousKey);
    }
  }, [previousKey, judgeLastMove]);

  // A full analysis (shown in progress, with Cancel), however long it takes, or for the
  // engine's move, until its time limit. If it fails or runs out of time, the engine pauses if
  // it was waiting for it, rather than trying again (and again).
  // An analysis carrying on from an earlier one which ran out of time starts from its time and
  // progress.
  const analyse = (kind: AnalysisKind, position: PositionData, key: string, carryOn?: TimedOut) => {
    const forEngine = engineActive;
    const timeLimit = forEngine ? timeLimitSeconds(engineTimeLimit) : undefined;
    const a = (analyser.current ??= new Analyser());
    const startedAt = performance.now();
    const start = startedAt - (carryOn?.elapsedMs ?? 0);
    let progress = carryOn?.progress;
    setRunning({ kind, key, start, progress });
    setNow(startedAt);
    a.run(kind, position, {
      timeLimit,
      onProgress: (p) => {
        progress = carryOn?.progress ? addProgress(carryOn.progress, p) : p;
        const latest = progress;
        setRunning((r) => (r && r.key === key && r.kind === kind ? { ...r, progress: latest } : r));
      },
    })
      .then((result) => {
        storeResult(key, kind, result!, start);
        setRunning(null);
      })
      .catch((e: unknown) => {
        if (e instanceof TimeLimitError) {
          setTimedOut({ key, seconds: e.seconds, elapsedMs: performance.now() - start, progress });
          setRunning(null);
          setPaused(true);
        } else if (!(e instanceof CancelledError)) {
          setError(errorMessage(e));
          setRunning(null);
          setBoardValues("off");
          if (forEngine) setPaused(true);
        }
      });
  };

  // When it's the engine's turn, evaluate the position (unless already cached), then play
  // one of the best moves after a short pause
  useEffect(() => {
    if (!engineActive || !shownPosition || !shownKey) return;
    const evaluation = results.get(shownKey)?.evaluate;
    if (!evaluation) {
      if (!running) {
        const carryOn = resumingFrom.current?.key === shownKey ? resumingFrom.current : undefined;
        resumingFrom.current = null;
        analyse("evaluate", shownPosition, shownKey, carryOn);
      }
      return;
    }
    const timer = setTimeout(() => {
      const move = chooseEngineMove(evaluation.result);
      if (move) setGame((g) => (g ? playMove(g, move, engine.makeMove) : g));
    }, ENGINE_DELAY_MS);
    return () => clearTimeout(timer);
  }, [engineActive, shownKey, results, running]);

  useEffect(() => {
    loadEngine()
      .then(() => {
        // Open the game in the link, if any (see share.ts)
        if (location.hash.length > 1) {
          try {
            const shared = decodeGame(location.hash, engine.makeMove);
            setGame(shared.game);
            // The view in a link applies to this visit only, not the saved preference
            if (shared.view) setView(shared.view);
            return;
          } catch (e) {
            setError(`Couldn't open the game in this link: ${errorMessage(e)}`);
          }
        }
        setGame(gameFrom(engine.newGame([[3, 3]])));
      })
      .catch((e: unknown) => setLoadError(errorMessage(e)));
  }, []);

  const hashChangeHandler = useRef<() => void>(() => {});
  useEffect(() => {
    const onHashChange = () => hashChangeHandler.current();
    window.addEventListener("hashchange", onHashChange);
    return () => window.removeEventListener("hashchange", onHashChange);
  }, []);

  // Keep the address bar a link to the current game, replacing the URL rather than adding
  // to the browser history at every move (as part of rendering, so it's never out of date)
  const writtenHash = useRef<string | null>(null);
  useLayoutEffect(() => {
    if (!game) return;
    const hash = encodeGame(game, view);
    writtenHash.current = hash;
    if (location.hash.slice(1) !== hash) history.replaceState(null, "", `#${hash}`);
  }, [game, view]);

  // Arrow keys and Home/End move through the game, unless typing in a form field
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      const t = e.target;
      if (t instanceof HTMLInputElement || t instanceof HTMLTextAreaElement || e.altKey || e.ctrlKey || e.metaKey) return;
      const delta: Record<string, number> = { ArrowLeft: -1, ArrowRight: 1, Home: -Infinity, End: Infinity };
      if (e.key in delta) {
        e.preventDefault();
        setGame((g) => (g ? goTo(g, g.cursor + delta[e.key]) : g));
        setPaused(true);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  if (loadError) {
    return <p class="error">Could not load the engine: {loadError}</p>;
  }
  if (!game) {
    return <p class="loading">Loading engine…</p>;
  }

  const snap = current(game);
  const atStart = game.cursor === 0;
  const last = showLastMove && snap.lastMove ? moveToLine(snap.lastMove) : null;
  const multiPart = snap.position.parts.length > 1;
  // Margins depend on the game's starting position, so boards don't change size during it.
  // Left and right margins are shared by all parts, so their squares/coins line up in columns.
  const partMargins = game.snapshots[0].position.parts.map((p) => boardMargins(p, view));
  const left = Math.max(...partMargins.map((m) => m.left));
  const right = Math.max(...partMargins.map((m) => m.right));
  const margins = partMargins.map((m) => ({ ...m, left, right }));
  const maxWidth = Math.max(...snap.position.parts.map((p, i) => naturalWidth(p, margins[i])));
  const key = positionKey(snap.position);
  const here = results.get(key) ?? {};
  let labels: MoveLabels | undefined;
  if (boardValues === "evaluation" && here.evaluate) {
    const scoreDiff = snap.scores[snap.toMove] - snap.scores[other(snap.toMove)];
    labels = evaluationLabels(here.evaluate.result, scoreDiff, countScore ? "margin" : "engine", perspective === "A" && snap.toMove === "B");
  } else if (boardValues === "nimstring" && here.nimstring) {
    labels = nimstringLabels(here.nimstring.result.moves, snap.position);
  }
  // Nimstring value of each part, shown beside it while Nimstring values are on the board
  const partNimbers = boardValues === "nimstring" && here.nimstring ? partValues(here.nimstring.result.components, snap.position.parts.length) : null;
  // The column for them is kept while they're selected, even while they're being calculated
  // (left blank), so the boards don't shift sideways each time they're recalculated
  const showPartNimbers = boardValues === "nimstring";

  // Choosing values for the board calculates them if they aren't already available,
  // cancelling any other analysis in progress
  const chooseBoardValues = (values: BoardValues) => {
    setBoardValues(values);
    setError(null);
    const kind: AnalysisKind | null = values === "evaluation" ? "evaluate" : values === "nimstring" ? "nimstring" : null;
    if (running && running.kind !== kind) {
      analyser.current?.cancel();
      setRunning(null);
    }
    if (kind && !here[kind] && !(running?.key === key && running.kind === kind)) {
      analyse(kind, snap.position, key);
    }
  };

  // Stop the analysis in progress, switching off the values it was calculating (and
  // pausing the engine, if it was thinking about its move)
  const cancelAnalysis = () => {
    analyser.current?.cancel();
    setRunning(null);
    setBoardValues("off");
    if (engineActive) setPaused(true);
  };

  // Let the engine carry on (after being paused, or running out of time)
  const resume = () => {
    // (If it ran out of time on the position shown, it carries on from where it got to)
    resumingFrom.current = timedOut?.key === key ? timedOut : null;
    setPaused(false);
    setError(null);
    setTimedOut(null);
  };
  // Whether to say the engine ran out of time: only while it's still paused on that position
  const showTimedOut = timedOut !== null && engineToMove && paused && timedOut.key === key;

  // Moving through the game pauses the engine, so it doesn't play over later moves
  const navigate = (cursor: number) => {
    setGame(goTo(game, cursor));
    setPaused(true);
  };

  const chooseEngineSides = (sides: Record<Player, boolean>) => {
    setEngineSides(sides);
    resume();
  };

  const anyCaptured = snap.position.parts.some((p, i) => snap.owners[i].some((row, y) => row.some((_, x) => isCoinCaptured(p, x, y) && !game.voids?.[i][y][x])));
  // The judgement of the last move ("?" or "??"), if the position before it has been evaluated
  const previousEvaluation = previousKey ? results.get(previousKey)?.evaluate : undefined;
  const judgement =
    judgeLastMove && previousSnapshot && snap.lastMove && previousEvaluation
      ? judgeMove(previousEvaluation.result, snap.lastMove, previousSnapshot.scores[previousSnapshot.toMove] - previousSnapshot.scores[other(previousSnapshot.toMove)])
      : null;
  const lastMark =
    judgement?.mark && previousSnapshot
      ? { mark: judgement.mark, title: `Not the best move: ${previousSnapshot.toMove} ${outcome(judgement.played)} instead of ${outcome(judgement.best)}` }
      : null;
  // The badge beside the last move, for a mistake: "?" or "??". (Not its value, even with
  // evaluation values on the board: its value belongs to the position before it, so showing it
  // alongside the values for the position now would invite misleading comparisons.)
  const lastBadge: LastMoveBadge | null =
    lastMark && judgement?.mark ? { text: judgement.mark, kind: judgement.mark === "??" ? "blunder" : "inaccuracy", title: lastMark.title } : null;
  // The last move can be checked on request when judging it wasn't cheap
  const canCheckMove = judgeLastMove && !!previousKey && !previousEvaluation && notCheap.has(`${previousKey} evaluate`);
  const checkingMove = !!running && running.key === previousKey && running.kind === "evaluate" && !engineActive;
  const checkMove = () => {
    if (previousSnapshot && previousKey) analyse("evaluate", previousSnapshot.position, previousKey);
  };
  // Stop checking the last move (leaving the values on the board as they were)
  const cancelCheck = () => {
    analyser.current?.cancel();
    setRunning(null);
  };

  // Canonical moves, when highlighted: the whole sequence of them which can be played from
  // here. Clicking any of them plays them all (see play).
  const canonicalPlan = showCanonical && !snap.gameOver ? canonicalSequence(snap.position, engine.canonicalMoves, engine.makeMove) : { moves: [], captured: [] };
  const canonical = new Set(canonicalPlan.moves.map(lineKeyOf));

  // The position split into its independent components, if that would give more parts
  const split = engine.split(snap.position);

  const attempt = (f: () => void) => {
    try {
      f();
      setError(null);
    } catch (e) {
      setError(errorMessage(e));
    }
  };

  // Start a new game: the engine stops playing, so it never starts on a game unasked
  // Stop the engine playing (setting "Engine plays" to Neither): done for every change to the
  // position which isn't a move, so the engine never starts playing a position unasked
  const stopEngine = () => {
    setEngineSides({ A: false, B: false });
    setPaused(false);
  };

  const startGame = (newGame: GameState) => {
    setGame(newGame);
    stopEngine();
  };

  // A different link pasted into the address bar (changing just the fragment, so the page
  // isn't reloaded) opens that game
  hashChangeHandler.current = () => {
    const hash = location.hash.slice(1);
    if (!hash || hash === writtenHash.current) return;
    try {
      const shared = decodeGame(hash, engine.makeMove);
      startGame(shared.game);
      if (shared.view) setView(shared.view);
      setPanel(null);
      setError(null);
    } catch (e) {
      setError(`Couldn't open the game in this link: ${errorMessage(e)}`);
    }
  };

  const load = (position: PositionData, moves: CPosMove[] = [], newName = "position") =>
    attempt(() => {
      startGame(gameFrom(position, moves));
      setName(newName);
      setPanel(null);
    });

  // Playing a highlighted canonical move plays the whole sequence of them: it's assumed that
  // whoever highlights them wants to play them
  const play = (line: Line) =>
    attempt(() => {
      setGame(canonical.has(lineKey(line)) ? playCanonicalMoves(game, engine.canonicalMoves, engine.makeMove) : playMove(game, lineToMove(line), engine.makeMove));
      setPaused(false);
    });

  // Setting up the starting position: change who owns a captured coin, or who moves first
  // Like any other change to the position which isn't a move, these stop the engine
  // playing, so it doesn't move before the start has been set up
  const cycleOwner = (part: number, x: number, y: number) =>
    attempt(() => {
      const owners = snap.owners.map((p, i) => (i === part ? p.map((row, ry) => row.map((o, rx) => (rx === x && ry === y ? nextOwner(o) : o))) : p));
      setGame(withStart(game, { owners }, engine.makeMove));
      stopEngine();
    });
  const swapFirstPlayer = () =>
    attempt(() => {
      setGame(withStart(game, { toMove: other(game.snapshots[0].toMove) }, engine.makeMove));
      stopEngine();
    });

  const togglePanel = (p: Panel) => setPanel(panel === p ? null : p);

  const chooseView = (v: View) => {
    setView(v);
    saveView(v);
  };

  const { A, B } = snap.scores;
  const status = snap.gameOver
    ? A === B
      ? `Game over: draw ${A}–${B}`
      : `Game over: ${A > B ? "A" : "B"} wins ${Math.max(A, B)}–${Math.min(A, B)}, by ${Math.abs(A - B)}`
    : engineToMove
      ? `${snap.toMove} (engine) ${paused ? "to move" : "thinking…"}`
      : `${snap.toMove} to move`;
  const lead = A === B ? "Scores level" : `${A > B ? "A" : "B"} leads by ${Math.abs(A - B)}`;

  return (
    <main>
      <header>
        <h1>Dots-and-Boxes Engine</h1>
      </header>

      <section class="toolbar" aria-label="Game setup">
        <div class="buttons">
          <button type="button" aria-expanded={panel === "new"} onClick={() => togglePanel("new")}>
            New game
          </button>
          <button
            type="button"
            disabled={atStart}
            title="Start a new game from the current position, keeping the scores"
            onClick={() => {
              startGame(startFromCurrent(game));
              setPanel(null);
            }}
          >
            Start from here
          </button>
          <button
            type="button"
            disabled={!split}
            title="Start a new game from the current position, with each independent component as a separate part"
            onClick={() => {
              if (split) startGame(startSplit(game, split));
              setPanel(null);
            }}
          >
            Split
          </button>
          <button type="button" aria-expanded={panel === "import"} onClick={() => togglePanel("import")}>
            Import
          </button>
          <button type="button" aria-expanded={panel === "export"} onClick={() => togglePanel("export")}>
            Export
          </button>
        </div>
      </section>

      {panel === "new" && (
        <NewGamePanel
          initial={sizesOf(snap.position)}
          onLoad={load}
          onRestart={
            game.snapshots.length > 1
              ? () => {
                  startGame(restart(game));
                  setPanel(null);
                }
              : undefined
          }
        />
      )}
      {panel === "import" && <ImportPanel onLoad={load} />}
      {panel === "export" && <ExportPanel position={snap.position} name={name} link={`${location.origin}${location.pathname}#${encodeGame(game, view)}`} />}

      <section class="status" aria-live="polite">
        <div class={`score player-A${!snap.gameOver && snap.toMove === "A" ? " active" : ""}`}>
          <span class="name">A</span> <span class="points">{A}</span>
        </div>
        <div class="message">
          {status}
          {atStart && !snap.gameOver && (
            <button type="button" class="quiet small" onClick={swapFirstPlayer}>
              Switch
            </button>
          )}
          {/* (Unless the message about running out of time is shown, with its own Resume button) */}
          {engineToMove && paused && !showTimedOut && (
            <button type="button" class="quiet small" onClick={resume}>
              Resume
            </button>
          )}
          {!snap.gameOver && <div class="lead">{lead}</div>}
        </div>
        <div class={`score player-B${!snap.gameOver && snap.toMove === "B" ? " active" : ""}`}>
          <span class="name">B</span> <span class="points">{B}</span>
        </div>
      </section>

      {error && <p class="error">{error}</p>}

      {/* All parts are drawn at the same scale: the board column fits the widest part (up
          to the maximum enlargement, or the available width), and each board's width is its
          natural width relative to the widest part's. Part numbers go on the left of compound
          positions, and Nimstring values (when shown) on the right. */}
      <section
        class="boards"
        style={{
          gridTemplateColumns: [
            multiPart && "var(--part-number-width)",
            `min(${MAX_ENLARGEMENT * maxWidth}px, calc(100%${multiPart ? " - var(--part-number-width) - var(--boards-gap)" : ""}${showPartNimbers ? " - var(--part-value-width) - var(--boards-gap)" : ""}))`,
            showPartNimbers && "var(--part-value-width)",
          ]
            .filter(Boolean)
            .join(" "),
        }}
      >
        {snap.position.parts.map((data, i) => [
          multiPart && (
            <div key={`n${i}`} class="part-number">
              <button
                type="button"
                class="part-chip"
                title={`Play part ${i} on its own`}
                aria-label={`Play part ${i} on its own`}
                aria-expanded={isolating === i}
                onClick={() => setIsolating(isolating === i ? null : i)}
              >
                {i}
              </button>
              {isolating === i && (
                <ConfirmPopover
                  question={`Play part ${i} on its own?`}
                  confirmLabel="Play"
                  onConfirm={() => {
                    setIsolating(null);
                    startGame(isolatePart(game, i));
                    setName(`${name}-part${i}`);
                    setPanel(null);
                  }}
                  onCancel={closeIsolating}
                />
              )}
            </div>
          ),
          <div key={`b${i}`} class="part-board">
            <Board
              data={data}
              part={i}
              owners={snap.owners[i]}
              view={view}
              toMove={snap.toMove}
              lastMove={last && last.part === i ? last : null}
              lastBadge={last && last.part === i ? lastBadge : null}
              lastMover={previousSnapshot?.toMove ?? null}
              interactive={!snap.gameOver && !engineActive}
              onPlay={play}
              onCycleOwner={atStart ? (x, y) => cycleOwner(i, x, y) : undefined}
              voids={game.voids?.[i]}
              canonical={canonical}
              canonicalCoins={canonicalPlan.captured.filter((c) => c.part === i).map((c): [number, number] => [c.x, c.y])}
              margins={margins[i]}
              width={`${(100 * naturalWidth(data, margins[i])) / maxWidth}%`}
              labels={labels}
            />
          </div>,
          showPartNimbers && (
            <div key={`v${i}`} class="part-value" title={multiPart ? `Nimstring value of part ${i}` : "Nimstring value"}>
              {partNimbers && <NimstringValueText value={partNimbers[i]} />}
            </div>
          ),
        ])}
        {showPartNimbers && multiPart && [
          <div key="tn" />,
          <div key="tl" class="total-label">
            Total
          </div>,
          <div key="tv" class="part-value total" title="Nimstring value of the whole position">
            {here.nimstring && <NimstringValueText value={here.nimstring.result.value} />}
          </div>,
        ]}
      </section>

      {atStart && anyCaptured && (
        <p class="hint">
          At the start of a game, click a captured {view === "dab" ? "square" : "coin"} to change who owns it.
        </p>
      )}

      <nav class="history" aria-label="Move history">
        <button type="button" onClick={() => navigate(0)} disabled={atStart} aria-label="Go to start">
          ⏮
        </button>
        <button type="button" onClick={() => navigate(game.cursor - 1)} disabled={atStart} aria-label="Back one move">
          ◀
        </button>
        <span class="move-count">
          Move {game.cursor} of {game.snapshots.length - 1}
          {lastMark && (
            <span class={`move-mark ${lastMark.mark === "??" ? "blunder" : "inaccuracy"}`} title={lastMark.title}>
              {lastMark.mark}
            </span>
          )}
          {canCheckMove && !running && (
            <button type="button" class="quiet small" title="Evaluate the position before the last move, to judge it" onClick={checkMove}>
              Check move
            </button>
          )}
          {checkingMove && running && (
            // (Just the time, to keep the history bar on one line: the rest is in the tooltip)
            <span class="checking" title={progressText(running.progress) ?? undefined}>
              Checking… {elapsedText(Math.max(0, now - running.start) / 1000)}
              <button type="button" class="quiet small" onClick={cancelCheck}>
                Cancel
              </button>
            </span>
          )}
        </span>
        <button type="button" onClick={() => navigate(game.cursor + 1)} disabled={game.cursor === game.snapshots.length - 1} aria-label="Forward one move">
          ▶
        </button>
        <button type="button" onClick={() => navigate(game.snapshots.length - 1)} disabled={game.cursor === game.snapshots.length - 1} aria-label="Go to end">
          ⏭
        </button>
      </nav>

      <section class="controls" aria-label="Settings">
        <Segmented
          label="Engine plays"
          options={[
            ["none", "Neither"],
            ["A", "A"],
            ["B", "B"],
            ["both", "Both"],
          ]}
          value={engineSides.A ? (engineSides.B ? "both" : "A") : engineSides.B ? "B" : "none"}
          onChange={(v) => chooseEngineSides({ A: v === "A" || v === "both", B: v === "B" || v === "both" })}
        />
        {running && engineActive && (
          <AnalysisProgress label={`${snap.toMove} (engine) thinking`} seconds={Math.max(0, now - running.start) / 1000} progress={running.progress} onCancel={cancelAnalysis} />
        )}
        {/* Where its progress was shown, if the engine ran out of time */}
        {showTimedOut && (
          <div class="running timed-out" role="status">
            <div>
              <span class="timed-out-message">
                {snap.toMove} (engine) ran out of time ({formatTimeLimit(timedOut.seconds)})
              </span>
              <button type="button" class="small" onClick={resume}>
                Resume
              </button>
            </div>
            <div class="note">Resume to give it another {formatTimeLimit(timedOut.seconds)}, carrying on from where it got to, or change its time limit in More settings.</div>
          </div>
        )}
        <AnalysisPanel
          running={running && { kind: running.kind, seconds: Math.max(0, now - running.start) / 1000, progress: running.progress }}
          engineThinking={engineActive}
          checkingMove={checkingMove}
          boardValues={boardValues}
          onBoardValues={chooseBoardValues}
          onCancel={cancelAnalysis}
        />
        <Segmented
          label="View"
          options={[
            ["dab", "Dots and boxes"],
            ["snc", "Strings and coins"],
          ]}
          value={view}
          onChange={chooseView}
        />
        <MoreSettings
        countScore={countScore}
        onCountScore={(count) => {
          setCountScore(count);
          saveCountScore(count);
        }}
        perspective={perspective}
        onPerspective={(p) => {
          setPerspectives(countScore ? { ...perspectives, withScore: p } : { ...perspectives, withoutScore: p });
          savePerspective(countScore, p);
        }}
        toMove={snap.toMove}
        engineTimeLimit={engineTimeLimit}
        onEngineTimeLimit={(limit) => {
          setEngineTimeLimit(limit);
          saveEngineTimeLimit(limit);
        }}
        >
          <Segmented
            label="Last move"
            options={[
              ["judge", "Show and judge"],
              ["show", "Show"],
              ["hide", "Hide"],
            ]}
            value={lastMoveDisplay}
            onChange={(display: LastMoveDisplay) => {
              setLastMoveDisplay(display);
              saveLastMoveDisplay(display);
            }}
          />
          <Segmented
            label="Canonical moves"
            options={[
              [true, "Show"],
              [false, "Hide"],
            ]}
            value={showCanonical}
            onChange={(show) => {
              setShowCanonical(show);
              saveShowCanonical(show);
            }}
          />
        </MoreSettings>
      </section>
    </main>
  );
}
