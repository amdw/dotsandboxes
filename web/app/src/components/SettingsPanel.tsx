// Rarely-changed settings, at the end of the settings under the board, hidden until opened
import type { ComponentChildren } from "preact";
import { useState } from "preact/hooks";
import { Segmented, type Perspective } from "./AnalysisPanel";
import type { Player } from "../game/state";

// The longest the engine may think about a move (in seconds) before giving up, or "none"
export type EngineTimeLimit = "10" | "30" | "60" | "300" | "none";
export const DEFAULT_ENGINE_TIME_LIMIT: EngineTimeLimit = "60";

const TIME_LIMITS: [EngineTimeLimit, string][] = [
  ["10", "10 s"],
  ["30", "30 s"],
  ["60", "1 min"],
  ["300", "5 min"],
  ["none", "No limit"],
];

export function isEngineTimeLimit(value: unknown): value is EngineTimeLimit {
  return TIME_LIMITS.some(([limit]) => limit === value);
}

// The time limit in seconds, if any
export function timeLimitSeconds(limit: EngineTimeLimit): number | undefined {
  return limit === "none" ? undefined : Number(limit);
}

// A time limit (in seconds) as shown, e.g. "1 min"
export function formatTimeLimit(seconds: number): string {
  return seconds % 60 === 0 ? `${seconds / 60} min` : `${seconds} s`;
}

interface MoreSettingsProps {
  // Whether evaluation values include the score so far (see AnalysisPanel)
  countScore: boolean;
  onCountScore: (count: boolean) => void;
  perspective: Perspective;
  onPerspective: (perspective: Perspective) => void;
  toMove: Player;
  engineTimeLimit: EngineTimeLimit;
  onEngineTimeLimit: (limit: EngineTimeLimit) => void;
  // Further settings rows, shown first
  children?: ComponentChildren;
}

// Rendered inside the .controls grid, so that its rows line up with the other settings
export function MoreSettings(props: MoreSettingsProps) {
  const [open, setOpen] = useState(false);
  return (
    <>
      <div class="more-settings">
        <button type="button" class="disclosure" aria-expanded={open} onClick={() => setOpen(!open)}>
          More settings
        </button>
      </div>
      {open && (
        <>
        {props.children}
        <Segmented
          label="Evaluation values"
          options={[
            [true, "Including score so far"],
            [false, "Excluding score so far"],
          ]}
          value={props.countScore}
          onChange={props.onCountScore}
        />
        <Segmented
          label="Values for"
          options={[
            ["A", "Player A"],
            // Naming the player to move, so it's always clear whose values these are
            ["mover", `Player to move (${props.toMove})`],
          ]}
          value={props.perspective}
          onChange={props.onPerspective}
        />
        <Segmented label="Engine time limit" options={TIME_LIMITS} value={props.engineTimeLimit} onChange={props.onEngineTimeLimit} />
        </>
      )}
    </>
  );
}
