import { beforeAll, describe, expect, it } from "vitest";
import { engine } from "../engine/engine";
import { evaluationLabels, formatNimstring, lineKeyOf, nimstringLabels, positionKey, signed } from "./analysis";
import { loadTestEngine } from "./testEngine";

beforeAll(loadTestEngine);

const example = (id: string) => engine.examples().find((e) => e.id === id)!;

describe("analysis", () => {
  it("formats values", () => {
    expect(signed(3)).toBe("+3");
    expect(signed(-2)).toBe("−2");
    expect(signed(0)).toBe("0");
    expect(formatNimstring({ Nimber: 4 })).toBe("*4");
    expect(formatNimstring("Loony")).toBe("\u263D\uFE0E");
  });

  it("labels moves with engine values or final margins, marking the best", () => {
    const position = example("berlekamp-ex3.1").position;
    const evaluation = engine.evaluate(position);
    const engineLabels = evaluationLabels(evaluation, 0, "engine");
    expect(engineLabels.size).toBe(evaluation.moves.length);
    const best = [...engineLabels.values()].filter((l) => l.best);
    expect(best.length).toBeGreaterThan(0);
    expect(best.every((l) => l.text === signed(evaluation.value))).toBe(true);
    // Berlekamp's answer for exercise 3.1 is (2, 1) Bottom, worth +3
    const answer = engineLabels.get(lineKeyOf({ part: 0, m: { x: 2, y: 1, side: "Bottom" } }))!;
    expect(answer).toEqual({ text: "+3", best: true });

    const marginLabels = evaluationLabels(evaluation, 2, "margin");
    expect(marginLabels.get(lineKeyOf({ part: 0, m: { x: 2, y: 1, side: "Bottom" } }))).toEqual({ text: "+5", best: true });
  });

  it("labels moves with Nimstring values", () => {
    const position = example("berlekamp-p50").position;
    const analysis = engine.nimstringAnalysis(position);
    expect(formatNimstring(analysis.value)).toBe("*7");
    const labels = nimstringLabels(analysis.moves);
    expect(labels.size).toBe(analysis.moves.length);
    expect(labels.get(lineKeyOf({ part: 0, m: { x: 0, y: 3, side: "Bottom" } }))!.text).toBe("*0");
  });

  it("keys positions by content", () => {
    expect(positionKey(engine.newGame([[2, 2]]))).toBe(positionKey(engine.newGame([[2, 2]])));
    expect(positionKey(engine.newGame([[2, 2]]))).not.toBe(positionKey(engine.newGame([[2, 3]])));
  });
});
