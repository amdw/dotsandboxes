import { useEffect, useRef, useState } from "preact/hooks";
import { Analyser, CancelledError } from "./engine/analyser";
import { engine, errorMessage, loadEngine } from "./engine/engine";
import type { CPosMove, Evaluation, NimstringAnalysis, NimstringValue, PositionData } from "./engine/types";
import { AnalysisPanel, type BoardValues, type Timed } from "./components/AnalysisPanel";
import { Board, boardMargins, naturalWidth, type View } from "./components/Board";
import { ExportPanel, ImportPanel, NewGamePanel } from "./components/SetupPanels";
import { evaluationLabels, formatNimstring, LOONY, nimstringLabels, partValues, positionKey, type AnalysisKind, type MoveLabels } from "./game/analysis";
import { isCoinCaptured, lineToMove, moveToLine, type Line } from "./game/lines";
import { current, goTo, newGameState, nextOwner, other, playMove, startFrom, withStart, type GameState } from "./game/state";

const VIEW_KEY = "dab-view";
const COUNT_SCORE_KEY = "dab-count-score";
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

function saveCountScore(count: boolean) {
  try {
    localStorage.setItem(COUNT_SCORE_KEY, String(count));
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

export function App() {
  const [game, setGame] = useState<GameState | null>(null);
  const [name, setName] = useState("position");
  const [loadError, setLoadError] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [view, setView] = useState<View>(loadView);
  const [panel, setPanel] = useState<Panel | null>(null);

  // Engine analysis: results are cached per position (keyed by positionKey)
  const analyser = useRef<Analyser | null>(null);
  const [results, setResults] = useState<Map<string, { evaluate?: Timed<Evaluation>; nimstring?: Timed<NimstringAnalysis> }>>(new Map());
  const [running, setRunning] = useState<{ kind: AnalysisKind; key: string; start: number } | null>(null);
  const [now, setNow] = useState(0);
  // Values are only calculated when chosen, since analysing larger positions takes a long
  // time, so this is off unless the chosen values are available for the position shown
  const [boardValues, setBoardValues] = useState<BoardValues>("off");
  const [countScore, setCountScore] = useState(loadCountScore);

  // Tick while an analysis runs, to show the time taken so far
  useEffect(() => {
    if (!running) return;
    const timer = setInterval(() => setNow(performance.now()), 100);
    return () => clearInterval(timer);
  }, [running]);

  useEffect(() => () => analyser.current?.cancel(), []);

  // When the position shown changes, stop analysing the previous one, and switch the board
  // values off unless they're already available for the new position
  const shownPosition = game ? current(game).position : null;
  const shownKey = shownPosition ? positionKey(shownPosition) : null;
  const wanted: AnalysisKind | null = boardValues === "evaluation" ? "evaluate" : boardValues === "nimstring" ? "nimstring" : null;
  useEffect(() => {
    if (running && running.key !== shownKey) {
      analyser.current?.cancel();
      setRunning(null);
    }
    if (wanted && shownKey && !results.get(shownKey)?.[wanted] && !(running?.key === shownKey && running.kind === wanted)) {
      setBoardValues("off");
    }
  }, [shownKey]);

  const analyse = (kind: AnalysisKind, position: PositionData, key: string) => {
    const a = (analyser.current ??= new Analyser());
    const start = performance.now();
    setRunning({ kind, key, start });
    setNow(start);
    const promise: Promise<Evaluation | NimstringAnalysis> = kind === "evaluate" ? a.evaluate(position) : a.nimstringAnalysis(position);
    promise
      .then((result) => {
        const timed = { result, seconds: (performance.now() - start) / 1000 };
        setResults((prev) => new Map(prev).set(key, { ...prev.get(key), [kind]: timed }));
        setRunning(null);
      })
      .catch((e: unknown) => {
        if (!(e instanceof CancelledError)) {
          setError(errorMessage(e));
          setRunning(null);
          setBoardValues("off");
        }
      });
  };

  useEffect(() => {
    loadEngine()
      .then(() => setGame(gameFrom(engine.newGame([[3, 3]]))))
      .catch((e: unknown) => setLoadError(errorMessage(e)));
  }, []);

  // Arrow keys and Home/End move through the game, unless typing in a form field
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      const t = e.target;
      if (t instanceof HTMLInputElement || t instanceof HTMLTextAreaElement || e.altKey || e.ctrlKey || e.metaKey) return;
      const delta: Record<string, number> = { ArrowLeft: -1, ArrowRight: 1, Home: -Infinity, End: Infinity };
      if (e.key in delta) {
        e.preventDefault();
        setGame((g) => (g ? goTo(g, g.cursor + delta[e.key]) : g));
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
  const last = snap.lastMove ? moveToLine(snap.lastMove) : null;
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
    labels = evaluationLabels(here.evaluate.result, scoreDiff, countScore ? "margin" : "engine");
  } else if (boardValues === "nimstring" && here.nimstring) {
    labels = nimstringLabels(here.nimstring.result.moves);
  }
  // Nimstring value of each part, shown beside it while Nimstring values are on the board
  const partNimbers = boardValues === "nimstring" && here.nimstring ? partValues(here.nimstring.result.components, snap.position.parts.length) : null;

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

  // Stop the analysis in progress, switching off the values it was calculating
  const cancelAnalysis = () => {
    analyser.current?.cancel();
    setRunning(null);
    setBoardValues("off");
  };

  const anyCaptured = snap.position.parts.some((p, i) => snap.owners[i].some((row, y) => row.some((_, x) => isCoinCaptured(p, x, y))));

  const attempt = (f: () => void) => {
    try {
      f();
      setError(null);
    } catch (e) {
      setError(errorMessage(e));
    }
  };

  const load = (position: PositionData, moves: CPosMove[] = [], newName = "position") =>
    attempt(() => {
      setGame(gameFrom(position, moves));
      setName(newName);
      setPanel(null);
    });

  const play = (line: Line) => attempt(() => setGame(playMove(game, lineToMove(line), engine.makeMove)));

  // Setting up the starting position: change who owns a captured coin, or who moves first
  const cycleOwner = (part: number, x: number, y: number) =>
    attempt(() => {
      const owners = snap.owners.map((p, i) => (i === part ? p.map((row, ry) => row.map((o, rx) => (rx === x && ry === y ? nextOwner(o) : o))) : p));
      setGame(withStart(game, { owners }, engine.makeMove));
    });
  const swapFirstPlayer = () => attempt(() => setGame(withStart(game, { toMove: other(game.snapshots[0].toMove) }, engine.makeMove)));

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
              setGame(startFrom(snap));
              setPanel(null);
            }}
          >
            Start from here
          </button>
          <button type="button" aria-expanded={panel === "import"} onClick={() => togglePanel("import")}>
            Import
          </button>
          <button type="button" aria-expanded={panel === "export"} onClick={() => togglePanel("export")}>
            Export
          </button>
        </div>
        <div class="segmented" role="radiogroup" aria-label="View">
          <button type="button" role="radio" aria-checked={view === "dab"} onClick={() => chooseView("dab")}>
            Dots and boxes
          </button>
          <button type="button" role="radio" aria-checked={view === "snc"} onClick={() => chooseView("snc")}>
            Strings and coins
          </button>
        </div>
      </section>

      {panel === "new" && <NewGamePanel initial={sizesOf(snap.position)} onLoad={load} />}
      {panel === "import" && <ImportPanel onLoad={load} />}
      {panel === "export" && <ExportPanel position={snap.position} name={name} />}

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
            `min(${MAX_ENLARGEMENT * maxWidth}px, calc(100%${multiPart ? " - var(--part-number-width) - var(--boards-gap)" : ""}${partNimbers ? " - var(--part-value-width) - var(--boards-gap)" : ""}))`,
            partNimbers && "var(--part-value-width)",
          ]
            .filter(Boolean)
            .join(" "),
        }}
      >
        {snap.position.parts.map((data, i) => [
          multiPart && (
            <div key={`n${i}`} class="part-number" aria-hidden="true">
              {i}
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
              interactive={!snap.gameOver}
              onPlay={play}
              onCycleOwner={atStart ? (x, y) => cycleOwner(i, x, y) : undefined}
              margins={margins[i]}
              width={`${(100 * naturalWidth(data, margins[i])) / maxWidth}%`}
              labels={labels}
            />
          </div>,
          partNimbers && (
            <div key={`v${i}`} class="part-value" title={multiPart ? `Nimstring value of part ${i}` : "Nimstring value"}>
              <NimstringValueText value={partNimbers[i]} />
            </div>
          ),
        ])}
        {partNimbers && multiPart && here.nimstring && [
          <div key="tn" />,
          <div key="tl" class="total-label">
            Total
          </div>,
          <div key="tv" class="part-value total" title="Nimstring value of the whole position">
            <NimstringValueText value={here.nimstring.result.value} />
          </div>,
        ]}
      </section>

      {atStart && anyCaptured && (
        <p class="hint">
          At the start of a game, click a captured {view === "dab" ? "square" : "coin"} to change who owns it.
        </p>
      )}

      <nav class="history" aria-label="Move history">
        <button type="button" onClick={() => setGame(goTo(game, 0))} disabled={atStart} aria-label="Go to start">
          ⏮
        </button>
        <button type="button" onClick={() => setGame(goTo(game, game.cursor - 1))} disabled={atStart} aria-label="Back one move">
          ◀
        </button>
        <span class="move-count">
          Move {game.cursor} of {game.snapshots.length - 1}
        </span>
        <button type="button" onClick={() => setGame(goTo(game, game.cursor + 1))} disabled={game.cursor === game.snapshots.length - 1} aria-label="Forward one move">
          ▶
        </button>
        <button type="button" onClick={() => setGame(goTo(game, game.snapshots.length - 1))} disabled={game.cursor === game.snapshots.length - 1} aria-label="Go to end">
          ⏭
        </button>
      </nav>

      <AnalysisPanel
        running={running && { kind: running.kind, seconds: Math.max(0, now - running.start) / 1000 }}
        boardValues={boardValues}
        onBoardValues={chooseBoardValues}
        countScore={countScore}
        onCountScore={(count) => {
          setCountScore(count);
          saveCountScore(count);
        }}
        onCancel={cancelAnalysis}
      />
    </main>
  );
}
