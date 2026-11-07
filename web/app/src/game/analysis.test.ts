import { beforeAll, describe, expect, it } from "vitest";
import { engine } from "../engine/engine";
import { evaluationLabels, formatNimstring, lineKeyOf, nimstringLabels, positionKey, signed, summariseEvaluation, uncapturedCoins } from "./analysis";
import { loadTestEngine } from "./testEngine";

beforeAll(loadTestEngine);

const example = (id: string) => engine.examples().find((e) => e.id === id)!;

describe("analysis", () => {
  it("formats values", () => {
    expect(signed(3)).toBe("+3");
    expect(signed(-2)).toBe("−2");
    expect(signed(0)).toBe("0");
    expect(formatNimstring({ Nimber: 4 })).toBe("*4");
    expect(formatNimstring("Loony")).toBe("Loony");
  });

  it("summarises an evaluation as engine value, margin and final scores", () => {
    // Two 5-chains: the player to move loses 2-8 (paper figure)
    const position = example("double-deal").position;
    const evaluation = engine.evaluate(position);
    expect(uncapturedCoins(position)).toBe(10);
    const fresh = summariseEvaluation(evaluation, position, "A", { A: 0, B: 0 });
    expect(fresh).toEqual({ mover: "A", value: -6, scoreDiff: 0, margin: -6, finalScores: { A: 2, B: 8 } });
    // With B to move and A already 3 ahead (from coins elsewhere), B still nets 6 more
    const ahead = summariseEvaluation(evaluation, position, "B", { A: 3, B: 0 });
    expect(ahead).toEqual({ mover: "B", value: -6, scoreDiff: -3, margin: -9, finalScores: { A: 11, B: 2 } });
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
