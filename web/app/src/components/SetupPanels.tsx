// Panels for choosing a starting position: a new game, a named example, or a .pos file
import { useState } from "preact/hooks";
import { engine, errorMessage } from "../engine/engine";
import type { CPosMove, Example, PositionData } from "../engine/types";

export const MAX_SIZE = 10;
const MAX_PARTS = 8;

export type LoadPosition = (position: PositionData, moves?: CPosMove[]) => void;

function clamp(n: number): number {
  return Number.isFinite(n) ? Math.max(1, Math.min(MAX_SIZE, Math.round(n))) : 1;
}

export function NewGamePanel({ initial, onLoad }: { initial: [number, number][]; onLoad: LoadPosition }) {
  const [sizes, setSizes] = useState<[number, number][]>(initial);
  const [error, setError] = useState<string | null>(null);

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
    <form class="panel new-game" onSubmit={start}>
      <p class="hint">
        Start from an empty board. Add more parts to play a compound position, with each part a separate board.
      </p>
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
  );
}

export function ExamplesPanel({ examples, onLoad }: { examples: Example[]; onLoad: (example: Example) => void }) {
  return (
    <div class="panel">
      <ul class="examples">
        {examples.map((example) => (
          <li key={example.id}>
            <button type="button" class="example" onClick={() => onLoad(example)}>
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
        Paste a position in the engine's <code>.pos</code> format, or choose a file. The first line gives the width and height
        of each part, followed by one move per line, e.g. <code>0 0 t</code> (or <code>1 0 0 t</code> for part 1).
      </p>
      <textarea rows={6} spellcheck={false} value={text} aria-label="Position text" onInput={(e) => setText(e.currentTarget.value)} placeholder={"3 2\n0 0 t\n1 0 t"} />
      <div class="panel-actions">
        <label class="file">
          <input type="file" accept=".pos,.txt,text/plain" onChange={onFile} />
          Choose file…
        </label>
        <button type="submit" class="primary" disabled={!text.trim()}>
          Load
        </button>
      </div>
      {error && <p class="error">{error}</p>}
    </form>
  );
}

// Offer the current position as a .pos file to save
export function downloadPos(position: PositionData, name: string) {
  const blob = new Blob([engine.writePos(position)], { type: "text/plain" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = `${name}.pos`;
  a.click();
  URL.revokeObjectURL(url);
}
