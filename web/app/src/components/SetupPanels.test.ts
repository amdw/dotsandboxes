import { describe, expect, it } from "vitest";
import { MAX_SIZE, parseSize } from "./SetupPanels";

describe("board size fields", () => {
  it("accept whole numbers in range, as typed", () => {
    expect(parseSize("1")).toBe(1);
    expect(parseSize(String(MAX_SIZE))).toBe(MAX_SIZE);
    expect(parseSize(" 7 ")).toBe(7);
    expect(parseSize("03")).toBe(3);
  });

  it("reject anything else, rather than changing it", () => {
    for (const text of ["", " ", "0", String(MAX_SIZE + 1), "13", "2.5", "-3", "abc", "3x"]) {
      expect(parseSize(text)).toBeNull();
    }
  });
});
