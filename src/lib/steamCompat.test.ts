import { describe, expect, it } from "vitest";
import { formatCompatSummary } from "./steamCompat";

const labels = {
  supported: "Supported",
  detected: "Detected",
  unknown: "version unknown",
  exeMissing: "steam.exe not found at {path}",
  unreadable: "version unreadable at {path}",
  warnOutside: "outside support window",
};

describe("formatCompatSummary", () => {
  it("includes supported and detected", () => {
    const { line, warn } = formatCompatSummary(
      {
        supportedSteam: "layout-x",
        supportedSteamNote: "note",
        detectedSteamVersion: "10.1.0.0",
        steamWithinSupport: true,
      },
      labels,
    );
    expect(line).toContain("layout-x");
    expect(line).toContain("10.1.0.0");
    expect(warn).toBe(false);
  });

  it("warns when outside support", () => {
    const { warn, line } = formatCompatSummary(
      {
        supportedSteam: "layout-x",
        supportedSteamNote: "note",
        detectedSteamVersion: "9.0.0.0",
        steamWithinSupport: false,
      },
      labels,
    );
    expect(warn).toBe(true);
    expect(line).toContain("outside support");
  });

  it("states exe missing with checked path", () => {
    const { line } = formatCompatSummary(
      {
        supportedSteam: "layout-x",
        supportedSteamNote: "note",
        detectedSteamVersion: null,
        steamWithinSupport: null,
        checkedExePath: "C:\\Steam\\steam.exe",
        detectionState: "exe_missing",
      },
      labels,
    );
    expect(line).toContain("steam.exe not found at C:\\Steam\\steam.exe");
  });
});
