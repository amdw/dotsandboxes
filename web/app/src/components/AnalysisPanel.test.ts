import { describe, expect, it } from "vitest";
import { elapsedText, progressText } from "./AnalysisPanel";

describe("elapsedText", () => {
  it("shows the time to a tenth of a second", () => {
    expect(elapsedText(0.44)).toBe("0.4 s");
    expect(elapsedText(40.06)).toBe("40.1 s");
  });
});

describe("progressText", () => {
  it("shows nothing until there is progress", () => {
    expect(progressText()).toBeNull();
    expect(progressText({ positions: 0, cached: 0, capacity: 100, forgotten: 0 })).toBeNull();
  });

  it("shows how full the cache is until the analysis has had to forget values", () => {
    expect(progressText({ positions: 1_400_000, cached: 1_750_000, capacity: 14_680_064, forgotten: 0 })).toBe("1.4M positions · cache 11% full");
    expect(progressText({ positions: 999, cached: 999, capacity: 1000, forgotten: 0 })).toBe("999 positions · cache 99% full");
  });

  it("then shows how many values it has forgotten", () => {
    expect(progressText({ positions: 21_000_000, cached: 9_000_000, capacity: 14_680_064, forgotten: 7_340_000 })).toBe("21.0M positions · cache full, 7.3M forgotten");
  });
});
