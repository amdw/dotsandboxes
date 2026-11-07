import { useEffect, useState } from "preact/hooks";
import { engine, errorMessage, loadEngine } from "./engine/engine";
import type { CPosMove, PositionData } from "./engine/types";
import { Board, boardMargins, naturalWidth, type View } from "./components/Board";
import { ExportPanel, ImportPanel, NewGamePanel } from "./components/SetupPanels";
import { isCoinCaptured, lineToMove, moveToLine, type Line } from "./game/lines";
import { current, goTo, newGameState, nextOwner, other, playMove, startFrom, withStart, type GameState } from "./game/state";

const VIEW_KEY = "dab-view";
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

export function App() {
  const [game, setGame] = useState<GameState | null>(null);
  const [name, setName] = useState("position");
  const [loadError, setLoadError] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [view, setView] = useState<View>(loadView);
  const [panel, setPanel] = useState<Panel | null>(null);

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
      : `Game over: ${A > B ? "A" : "B"} wins ${Math.max(A, B)}–${Math.min(A, B)}`
    : `${snap.toMove} to move`;

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
        </div>
        <div class={`score player-B${!snap.gameOver && snap.toMove === "B" ? " active" : ""}`}>
          <span class="name">B</span> <span class="points">{B}</span>
        </div>
      </section>

      {error && <p class="error">{error}</p>}

      {/* All parts are drawn at the same scale: the board column fits the widest part (up
          to the maximum enlargement, or the available width), and each board's width is its
          natural width relative to the widest part's. */}
      <section
        class={multiPart ? "boards multi" : "boards"}
        style={{
          gridTemplateColumns: multiPart
            ? `var(--part-number-width) min(${MAX_ENLARGEMENT * maxWidth}px, calc(100% - var(--part-number-width) - var(--boards-gap)))`
            : `min(${MAX_ENLARGEMENT * maxWidth}px, 100%)`,
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
            />
          </div>,
        ])}
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
    </main>
  );
}
