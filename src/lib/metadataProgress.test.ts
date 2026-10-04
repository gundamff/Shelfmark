import { describe, expect, it } from "vitest";
import { coveragePercent } from "./metadataProgress";

describe("coveragePercent", () => {
  it("handles empty library", () => {
    expect(coveragePercent(0, 0)).toBe(0);
  });

  it("rounds percent", () => {
    expect(coveragePercent(1, 3)).toBe(33);
    expect(coveragePercent(40, 40)).toBe(100);
  });
});
