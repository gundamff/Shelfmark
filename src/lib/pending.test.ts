import { describe, expect, it } from "vitest";
import { formatPendingSummary, isPendingEmpty } from "./pending";

describe("pending helpers", () => {
  it("formats counts", () => {
    expect(
      formatPendingSummary({
        create: [1],
        rename: [],
        delete: [],
        add_members: [1, 2],
        remove_members: [],
      }),
    ).toBe("1 create · 0 rename · 0 delete · 2 add-ops · 0 remove-ops");
  });

  it("detects empty pending", () => {
    expect(
      isPendingEmpty({
        create: [],
        rename: [],
        delete: [],
        add_members: [],
        remove_members: [],
      }),
    ).toBe(true);
  });
});
