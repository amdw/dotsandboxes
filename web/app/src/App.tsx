import { useEffect, useState } from "preact/hooks";
import { engine, errorMessage, loadEngine } from "./engine/engine";
import type { CPosMove, Example, PositionData } from "./engine/types";
import { Board, type View } from "./components/Board";
import { downloadPos, ExamplesPanel, ImportPanel, NewGamePanel } from "./components/SetupPanels";
import { lineToMove, moveToLine, toggleString, type Line } from "./game/lines";
import { current, goTo, newGameState, playMove, type GameState } from "./game/state";

const VIEW_KEY = "dab-view";

type Panel = "new" | "examples" | "import";

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
  // While editing, the position being set up
  const [draft, setDraft] = useState<PositionData | null>(null);

  useEffect(() => {
    loadEngine()
      .then(() => setGame(gameFrom(engine.newGame([[3, 3]]))))
      .catch((e: unknown) => setLoadError(errorMessage(e)));
  }, []);

  // Arrow keys and Home/End move through the game, unless typing in a form field or editing
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      const t = e.target;
      if (t instanceof HTMLInputElement || t instanceof HTMLTextAreaElement || e.altKey || e.ctrlKey || e.metaKey || draft) return;
      const delta: Record<string, number> = { ArrowLeft: -1, ArrowRight: 1, Home: -Infinity, End: Infinity };
      if (e.key in delta) {
        e.preventDefault();
        setGame((g) => (g ? goTo(g, g.cursor + delta[e.key]) : g));
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [draft]);

  if (loadError) {
    return <p class="error">Could not load the engine: {loadError}</p>;
  }
  if (!game) {
    return <p class="loading">Loading engine…</p>;
  }

  const snap = current(game);
  const editing = draft !== null;
  const position = draft ?? snap.position;
  const last = !editing && snap.lastMove ? moveToLine(snap.lastMove) : null;
  const multiPart = position.parts.length > 1;

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

  const loadExample = (example: Example) => load(example.position, example.moves, example.id);

  const onLine = (line: Line) =>
    attempt(() => {
      if (draft) {
        setDraft(toggleString(draft, line));
      } else {
        setGame(playMove(game, lineToMove(line), engine.makeMove));
      }
    });

  const togglePanel = (p: Panel) => setPanel(panel === p ? null : p);

  const startEditing = () => {
    setPanel(null);
    setDraft(snap.position);
  };

  const finishEditing = () => {
    if (draft) load(draft);
    setDraft(null);
  };

  const chooseView = (v: View) => {
    setView(v);
    saveView(v);
  };

  const viewToggle = (
    <div class="segmented" role="radiogroup" aria-label="View">
      <button type="button" role="radio" aria-checked={view === "dab"} onClick={() => chooseView("dab")}>
        Dots and boxes
      </button>
      <button type="button" role="radio" aria-checked={view === "snc"} onClick={() => chooseView("snc")}>
        Strings and coins
      </button>
    </div>
  );

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

      {editing ? (
        <section class="toolbar editbar" aria-label="Edit position">
          <p class="hint">
            {view === "dab" ? "Click lines to draw or erase them." : "Click strings to cut or restore them."} Edits are not
            scored; when you're done, a new game starts from the position.
          </p>
          <div class="buttons">
            {viewToggle}
            <button type="button" onClick={() => attempt(() => setDraft(engine.newGame(sizesOf(position))))}>
              Empty board
            </button>
            <button type="button" onClick={() => setDraft(null)}>
              Cancel
            </button>
            <button type="button" class="primary" onClick={finishEditing}>
              Done
            </button>
          </div>
        </section>
      ) : (
        <section class="toolbar" aria-label="Game setup">
          <div class="buttons">
            <button type="button" aria-expanded={panel === "new"} onClick={() => togglePanel("new")}>
              New game
            </button>
            <button type="button" aria-expanded={panel === "examples"} onClick={() => togglePanel("examples")}>
              Examples
            </button>
            <button type="button" onClick={startEditing}>
              Edit position
            </button>
            <button type="button" aria-expanded={panel === "import"} onClick={() => togglePanel("import")}>
              Import
            </button>
            <button type="button" onClick={() => attempt(() => downloadPos(snap.position, name))}>
              Export
            </button>
          </div>
          {viewToggle}
        </section>
      )}

      {!editing && panel === "new" && <NewGamePanel initial={sizesOf(snap.position)} onLoad={load} />}
      {!editing && panel === "examples" && <ExamplesPanel examples={engine.examples()} onLoad={loadExample} />}
      {!editing && panel === "import" && <ImportPanel onLoad={load} />}

      {!editing && (
        <section class="status" aria-live="polite">
          <div class={`score player-A${!snap.gameOver && snap.toMove === "A" ? " active" : ""}`}>
            <span class="name">A</span> <span class="points">{A}</span>
          </div>
          <div class="message">{status}</div>
          <div class={`score player-B${!snap.gameOver && snap.toMove === "B" ? " active" : ""}`}>
            <span class="name">B</span> <span class="points">{B}</span>
          </div>
        </section>
      )}

      {error && <p class="error">{error}</p>}

      <section class="boards">
        {position.parts.map((data, i) => (
          <figure key={i}>
            <Board
              data={data}
              part={i}
              owners={editing ? Array.from({ length: data.height }, () => Array<null>(data.width).fill(null)) : snap.owners[i]}
              view={view}
              mode={editing ? "edit" : "play"}
              toMove={snap.toMove}
              lastMove={last && last.part === i ? last : null}
              interactive={editing || !snap.gameOver}
              onPlay={onLine}
            />
            {multiPart && <figcaption>Part {i}</figcaption>}
          </figure>
        ))}
      </section>

      {!editing && (
        <nav class="history" aria-label="Move history">
          <button type="button" onClick={() => setGame(goTo(game, 0))} disabled={game.cursor === 0} aria-label="Go to start">
            ⏮
          </button>
          <button type="button" onClick={() => setGame(goTo(game, game.cursor - 1))} disabled={game.cursor === 0} aria-label="Back one move">
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
      )}
    </main>
  );
}
