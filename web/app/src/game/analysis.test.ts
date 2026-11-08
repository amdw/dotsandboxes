import { beforeAll, describe, expect, it } from "vitest";
import { engine } from "../engine/engine";
import { chooseEngineMove, evaluationLabels, formatNimstring, isWinningNimstringMove, lineKeyOf, nimAdd, nimstringLabels, partValues, positionKey, signed } from "./analysis";
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
    const labels = nimstringLabels(analysis.moves, position);
    expect(labels.size).toBe(analysis.moves.length);
    expect(labels.get(lineKeyOf({ part: 0, m: { x: 0, y: 3, side: "Bottom" } }))).toEqual({ text: "*0", best: true });
    // The position is *7, so the winning moves are exactly the three (non-capturing) moves to *0
    const winning = [...labels.values()].filter((l) => l.best);
    expect(winning).toHaveLength(3);
    expect(winning.every((l) => l.text === "*0")).toBe(true);
  });

  it("decides which Nimstring moves win, depending on whether they capture", () => {
    expect(isWinningNimstringMove(false, { Nimber: 0 })).toBe(true);
    expect(isWinningNimstringMove(false, { Nimber: 2 })).toBe(false);
    expect(isWinningNimstringMove(false, "Loony")).toBe(false);
    expect(isWinningNimstringMove(true, { Nimber: 0 })).toBe(false);
    expect(isWinningNimstringMove(true, { Nimber: 3 })).toBe(true);
    expect(isWinningNimstringMove(true, "Loony")).toBe(true);

    // An opened 3-chain with another closed 3-chain: taking the first coin leaves a loony
    // position for the capturer, who moves again, so it's a winning capture
    const nim = example("nimstring-examples").position.parts;
    const opened = engine.makeMove({ parts: [nim[5], nim[5]] }, { part: 0, m: { x: 0, y: 0, side: "Left" } }).position;
    const labels = nimstringLabels(engine.nimstringAnalysis(opened).moves, opened);
    expect(labels.get(lineKeyOf({ part: 0, m: { x: 0, y: 0, side: "Right" } }))!.best).toBe(true);
  });

  it("keys positions by content", () => {
    expect(positionKey(engine.newGame([[2, 2]]))).toBe(positionKey(engine.newGame([[2, 2]])));
    expect(positionKey(engine.newGame([[2, 2]]))).not.toBe(positionKey(engine.newGame([[2, 3]])));
  });

  it("adds Nimstring values", () => {
    expect(nimAdd({ Nimber: 5 }, { Nimber: 3 })).toEqual({ Nimber: 6 });
    expect(nimAdd({ Nimber: 2 }, "Loony")).toBe("Loony");
  });

  it("gives the value of each part from its components, adding up to the whole", () => {
    // The page 50 example as one part with three components, plus two more parts
    const p50 = example("berlekamp-p50").position.parts[0];
    const nim = example("nimstring-examples").position.parts;
    const position = { parts: [p50, nim[4], nim[5]] };
    const analysis = engine.nimstringAnalysis(position);
    const values = partValues(analysis.components, 3);
    expect(values).toEqual([{ Nimber: 7 }, { Nimber: 1 }, { Nimber: 0 }]);
    expect(values.reduce(nimAdd)).toEqual(analysis.value);
    // A loony part makes the whole position loony
    const loony = engine.nimstringAnalysis({ parts: [nim[2], nim[4]] });
    expect(partValues(loony.components, 2)).toEqual(["Loony", { Nimber: 1 }]);
    expect(loony.value).toBe("Loony");
  });

  it("values finished parts as *0", () => {
    const finished = engine.newGame([[1, 1]]);
    let position = finished;
    for (const m of engine.legalMoves(finished)) position = engine.makeMove(position, m).position;
    const analysis = engine.nimstringAnalysis({ parts: [position.parts[0], example("nimstring-examples").position.parts[4]] });
    expect(partValues(analysis.components, 2)).toEqual([{ Nimber: 0 }, { Nimber: 1 }]);
  });

  it("chooses one of the best moves for the engine", () => {
    // Exercise 3.1 has a single best move, (2, 1) Bottom
    const evaluation = engine.evaluate(example("berlekamp-ex3.1").position);
    for (const r of [0, 0.5, 0.999]) {
      expect(chooseEngineMove(evaluation, () => r)).toEqual({ part: 0, m: { x: 2, y: 1, side: "Bottom" } });
    }
    // Two closed 3-chains: every move is equally good, and the choice spreads over them
    const twoChains = engine.evaluate({ parts: [example("nimstring-examples").position.parts[5], example("nimstring-examples").position.parts[5]] });
    const chosen = new Set([0, 0.3, 0.6, 0.99].map((r) => lineKeyOf(chooseEngineMove(twoChains, () => r)!)));
    expect(chosen.size).toBeGreaterThan(1);
    for (const key of chosen) {
      expect(twoChains.moves.find((m) => lineKeyOf(m.move) === key)!.value).toBe(twoChains.value);
    }
    // No moves left
    expect(chooseEngineMove({ value: 0, moves: [] })).toBeNull();
  });
});

