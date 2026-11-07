import { useEffect, useState } from "preact/hooks";
import { engine, loadEngine } from "./engine/engine";
import { Board, type View } from "./components/Board";
import { lineToMove, moveToLine, type Line } from "./game/lines";
import { current, goTo, newGameState, playMove, type GameState } from "./game/state";

const MAX_SIZE = 10;
const VIEW_KEY = "dab-view";

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

export function App() {
  const [game, setGame] = useState<GameState | null>(null);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [view, setView] = useState<View>(loadView);
  const [width, setWidth] = useState(3);
  const [height, setHeight] = useState(3);

  useEffect(() => {
    loadEngine()
      .then(() => setGame(newGameState(engine.newGame([[3, 3]]))))
      .catch((e: unknown) => setLoadError(String(e)));
  }, []);

  // Arrow keys and Home/End move through the game, unless typing in a form field
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.target instanceof HTMLInputElement || e.altKey || e.ctrlKey || e.metaKey) return;
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
  const last = snap.lastMove ? moveToLine(snap.lastMove) : null;
  const multiPart = snap.position.parts.length > 1;

  const play = (line: Line) => {
    try {
      setGame(playMove(game, lineToMove(line), engine.makeMove));
      setError(null);
    } catch (e) {
      setError(String(e));
    }
  };

  const startNewGame = (e: Event) => {
    e.preventDefault();
    try {
      setGame(newGameState(engine.newGame([[width, height]])));
      setError(null);
    } catch (err) {
      setError(String(err));
    }
  };

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
        <form class="new-game" onSubmit={startNewGame}>
          <label>
            Width
            <input type="number" min={1} max={MAX_SIZE} value={width} onInput={(e) => setWidth(clamp(e.currentTarget.valueAsNumber))} />
          </label>
          <label>
            Height
            <input type="number" min={1} max={MAX_SIZE} value={height} onInput={(e) => setHeight(clamp(e.currentTarget.valueAsNumber))} />
          </label>
          <button type="submit">New game</button>
        </form>
        <div class="segmented" role="radiogroup" aria-label="View">
          <button type="button" role="radio" aria-checked={view === "dab"} onClick={() => chooseView("dab")}>
            Dots and boxes
          </button>
          <button type="button" role="radio" aria-checked={view === "snc"} onClick={() => chooseView("snc")}>
            Strings and coins
          </button>
        </div>
      </section>

      <section class="status" aria-live="polite">
        <div class={`score player-A${!snap.gameOver && snap.toMove === "A" ? " active" : ""}`}>
          <span class="name">A</span> <span class="points">{A}</span>
        </div>
        <div class="message">{status}</div>
        <div class={`score player-B${!snap.gameOver && snap.toMove === "B" ? " active" : ""}`}>
          <span class="name">B</span> <span class="points">{B}</span>
        </div>
      </section>

      {error && <p class="error">{error}</p>}

      <section class="boards">
        {snap.position.parts.map((data, i) => (
          <figure key={i}>
            <Board
              data={data}
              part={i}
              owners={snap.owners[i]}
              view={view}
              toMove={snap.toMove}
              lastMove={last && last.part === i ? last : null}
              interactive={!snap.gameOver}
              onPlay={play}
            />
            {multiPart && <figcaption>Part {i}</figcaption>}
          </figure>
        ))}
      </section>

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
    </main>
  );
}

function clamp(n: number): number {
  return Number.isFinite(n) ? Math.max(1, Math.min(MAX_SIZE, Math.round(n))) : 1;
}
