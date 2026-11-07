// Panels for starting a new game (from an empty board or a named example), and for
// importing and exporting positions in the engine's .pos text format
import { useEffect, useRef, useState } from "preact/hooks";
import { engine, errorMessage } from "../engine/engine";
import type { CPosMove, Example, PositionData } from "../engine/types";

export const MAX_SIZE = 10;
const MAX_PARTS = 8;

export type LoadPosition = (position: PositionData, moves?: CPosMove[], name?: string) => void;

function clamp(n: number): number {
  return Number.isFinite(n) ? Math.max(1, Math.min(MAX_SIZE, Math.round(n))) : 1;
}

export function NewGamePanel({ initial, onLoad }: { initial: [number, number][]; onLoad: LoadPosition }) {
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

      <h2>Examples</h2>
      <ul class="examples">
        {examples.map((example) => (
          <li key={example.id}>
            <button type="button" class="example" onClick={() => onLoad(example.position, example.moves, example.id)}>
              <span class="example-title">
                {example.title}
                {example.moves.length > 0 && <span class="tag">game, {example.moves.length} moves</span>}
              </span>
              <span class="example-description">{example.description}</span>
            </button>
          </li>
        ))}
      </ul>
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

export function ExportPanel({ position, name }: { position: PositionData; name: string }) {
  const text = engine.writePos(position);
  const area = useRef<HTMLTextAreaElement>(null);
  const [status, setStatus] = useState<string | null>(null);

  useEffect(() => setStatus(null), [text]);

  const copy = async () => {
    try {
      await navigator.clipboard.writeText(text);
      setStatus("Copied to the clipboard.");
    } catch {
      // The clipboard API needs a secure context (HTTPS or localhost) and permission
      area.current?.select();
      setStatus("Couldn't copy automatically: the text is selected, so copy it with Ctrl+C (or ⌘C).");
    }
  };

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
      <p class="hint">
        The current position in the engine's <code>.pos</code> format, which can be imported again here or loaded by the
        command-line engine.
      </p>
      <textarea ref={area} rows={Math.min(10, text.split("\n").length)} readOnly spellcheck={false} value={text} aria-label="Exported position text" />
      <div class="panel-actions">
        <button type="button" class="primary" onClick={copy}>
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
