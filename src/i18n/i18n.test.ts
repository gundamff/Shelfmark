import { describe, expect, it } from "vitest";
import { en } from "./en";
import { localizeError } from "./errors";
import { resolveLocale, t } from "./index";
import { zh } from "./zh";

describe("resolveLocale", () => {
  it("uses explicit en/zh over system", () => {
    expect(resolveLocale("en", "zh-CN")).toBe("en");
    expect(resolveLocale("zh", "en-US")).toBe("zh");
  });

  it("maps system zh* to zh, else en", () => {
    expect(resolveLocale("system", "zh-CN")).toBe("zh");
    expect(resolveLocale("system", "zh")).toBe("zh");
    expect(resolveLocale("system", "en-US")).toBe("en");
    expect(resolveLocale(null, "zh-TW")).toBe("zh");
    expect(resolveLocale(undefined, "fr-FR")).toBe("en");
  });
});

describe("dictionaries", () => {
  it("en and zh share the same keys", () => {
    expect(Object.keys(zh).sort()).toEqual(Object.keys(en).sort());
  });
});

describe("t", () => {
  it("interpolates params", () => {
    expect(t("en", "gamesShowing", { shown: 2, total: 10 })).toBe(
      "Showing 2 / 10",
    );
    expect(t("zh", "gamesShowing", { shown: 2, total: 10 })).toContain("2");
  });
});

describe("localizeError", () => {
  it("maps known fragments in zh", () => {
    expect(localizeError("zh", "no session open")).toMatch(/会话|未打开/);
    expect(
      localizeError("zh", "export_session: close Steam before export"),
    ).toMatch(/Steam|运行/);
  });

  it("keeps unknown text", () => {
    expect(localizeError("en", "weird boom 123")).toBe("weird boom 123");
  });
});
