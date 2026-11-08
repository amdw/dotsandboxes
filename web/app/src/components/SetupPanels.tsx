// Panels for starting a new game (restarting the current one, or from an empty board or
// a named example), and for importing and exporting positions in the engine's .pos text format
import { useEffect, useRef, useState } from "preact/hooks";
import { engine, errorMessage } from "../engine/engine";
import type { CPosMove, Example, PositionData } from "../engine/types";

export const MAX_SIZE = 10;
const MAX_PARTS = 8;

export type LoadPosition = (position: PositionData, moves?: CPosMove[], name?: string) => void;

function clamp(n: number): number {
  return Number.isFinite(n) ? Math.max(1, Math.min(MAX_SIZE, Math.round(n))) : 1;
}

interface NewGamePanelProps {
  initial: [number, number][];
  onLoad: LoadPosition;
  // Restart the current game from its starting position, if any moves have been played
  onRestart?: () => void;
}

function ExampleList({ examples, onLoad }: { examples: Example[]; onLoad: LoadPosition }) {
  return (
    <ul class="examples">
      {examples.map((example) => (
        <li key={example.id}>
          <button type="button" class="example" onClick={() => onLoad(example.position, example.moves, example.id)}>
            <span class="example-title">
              {example.title}
              {example.moves.length > 0 && <span class="tag">{example.moves.length} moves</span>}
            </span>
            <span class="example-description">{example.description}</span>
          </button>
        </li>
      ))}
    </ul>
  );
}

export function NewGamePanel({ initial, onLoad, onRestart }: NewGamePanelProps) {
  const [sizes, setSizes] = useState<[number, number][]>(initial);
  const [error, setError] = useState<string | null>(null);
  const examples: Example[] = engine.examples();

  const update = (i: number, dim: 0 | 1, value: number) =>
    setSizes(sizes.map((s, j) => (j === i ? (dim === 0 ? [clamp(value), s[1]] : [s[0], clamp(value)]) : s)));

  const start = (e: Event) => {
    e.preventDefault();
    try {
      onLoad(engine.newGame(sizes));
    } catch (err) {
      setError(errorMessage(err));
    }
  };

  return (
    <div class="panel new-game">
      {onRestart && (
        <div class="restart">
          <button type="button" class="primary" onClick={onRestart}>
            Restart this game
          </button>
          <p class="hint">Back to its starting position, discarding the moves played since.</p>
        </div>
      )}

      <form onSubmit={start}>
        <h2>Empty board</h2>
        <p class="hint">Add more parts to play a compound position, with each part a separate board.</p>
        {sizes.map(([w, h], i) => (
          <div class="part-size" key={i}>
            {sizes.length > 1 && <span class="part-label">Part {i}</span>}
            <label>
              Width
              <input type="number" min={1} max={MAX_SIZE} value={w} onInput={(e) => update(i, 0, e.currentTarget.valueAsNumber)} />
            </label>
            <label>
              Height
              <input type="number" min={1} max={MAX_SIZE} value={h} onInput={(e) => update(i, 1, e.currentTarget.valueAsNumber)} />
            </label>
            {sizes.length > 1 && (
              <button type="button" class="quiet" aria-label={`Remove part ${i}`} onClick={() => setSizes(sizes.filter((_, j) => j !== i))}>
                Remove
              </button>
            )}
          </div>
        ))}
        <div class="panel-actions">
          <button type="button" class="quiet" disabled={sizes.length >= MAX_PARTS} onClick={() => setSizes([...sizes, sizes[sizes.length - 1]])}>
            Add part
          </button>
          <button type="submit" class="primary">
            Start
          </button>
        </div>
        {error && <p class="error">{error}</p>}
      </form>

      <h2>Example positions</h2>
      <ExampleList examples={examples.filter((e) => e.moves.length === 0)} onLoad={onLoad} />

      <h2>Example games</h2>
      <p class="hint">Games which load at the start, ready to step through.</p>
      <ExampleList examples={examples.filter((e) => e.moves.length > 0)} onLoad={onLoad} />
    </div>
  );
}

export function ImportPanel({ onLoad }: { onLoad: LoadPosition }) {
  const [text, setText] = useState("");
  const [error, setError] = useState<string | null>(null);

  const load = (input: string) => {
    try {
      onLoad(engine.parsePos(input));
    } catch (err) {
      setError(errorMessage(err));
    }
  };

  const onFile = async (e: Event) => {
    const file = (e.currentTarget as HTMLInputElement).files?.[0];
    if (file) {
      const contents = await file.text();
      setText(contents);
      load(contents);
    }
  };

  return (
    <form
      class="panel import"
      onSubmit={(e) => {
        e.preventDefault();
        load(text);
      }}
    >
      <p class="hint">
        Paste a position in the engine's <code>.pos</code> format. The first line gives the width and height of each part,
        followed by one move per line, e.g. <code>0 0 t</code> (or <code>1 0 0 t</code> for part 1).
      </p>
      <textarea rows={6} spellcheck={false} value={text} aria-label="Position text" onInput={(e) => setText(e.currentTarget.value)} placeholder={"3 2\n0 0 t\n1 0 t"} />
      <div class="panel-actions">
        <button type="submit" class="primary" disabled={!text.trim()}>
          Load
        </button>
        <label class="file">
          <input type="file" accept=".pos,.txt,text/plain" onChange={onFile} />
          Load from file…
        </label>
      </div>
      {error && <p class="error">{error}</p>}
    </form>
  );
}

// Copy text to the clipboard, or if that isn't possible (the clipboard API needs a secure
// context and permission), select it in the given field so the user can copy it themselves.
// Returns a message saying which.
async function copyText(text: string, field: HTMLInputElement | HTMLTextAreaElement | null): Promise<string> {
  try {
    await navigator.clipboard.writeText(text);
    return "Copied to the clipboard.";
  } catch {
    field?.select();
    return "Couldn't copy automatically: the text is selected, so copy it with Ctrl+C (or ⌘C).";
  }
}

export function ExportPanel({ position, name, link }: { position: PositionData; name: string; link: string }) {
  const text = engine.writePos(position);
  const linkField = useRef<HTMLInputElement>(null);
  const area = useRef<HTMLTextAreaElement>(null);
  const [linkStatus, setLinkStatus] = useState<string | null>(null);
  const [status, setStatus] = useState<string | null>(null);

  useEffect(() => setLinkStatus(null), [link]);
  useEffect(() => setStatus(null), [text]);

  const download = () => {
    const url = URL.createObjectURL(new Blob([text], { type: "text/plain" }));
    const a = document.createElement("a");
    a.href = url;
    a.download = `${name}.pos`;
    a.click();
    URL.revokeObjectURL(url);
  };

  return (
    <div class="panel export">
      <h2>Link to this game</h2>
      <p class="hint">
        The whole game, from its starting position, including the moves played and the move shown. The address bar always
        holds this link too.
      </p>
      <input ref={linkField} class="link" type="text" readOnly value={link} aria-label="Link to this game" onFocus={(e) => e.currentTarget.select()} />
      <div class="panel-actions">
        <button type="button" class="primary" onClick={async () => setLinkStatus(await copyText(link, linkField.current))}>
          Copy link
        </button>
      </div>
      {linkStatus && (
        <p class="hint" role="status">
          {linkStatus}
        </p>
      )}

      <h2>Position file</h2>
      <p class="hint">
        The current position in the engine's <code>.pos</code> format, which can be imported again here or loaded by the
        command-line engine.
      </p>
      <textarea ref={area} rows={Math.min(10, text.split("\n").length)} readOnly spellcheck={false} value={text} aria-label="Exported position text" />
      <div class="panel-actions">
        <button type="button" class="primary" onClick={async () => setStatus(await copyText(text, area.current))}>
          Copy
        </button>
        <button type="button" onClick={download}>
          Download
        </button>
      </div>
      {status && (
        <p class="hint" role="status">
          {status}
        </p>
      )}
    </div>
  );
}
