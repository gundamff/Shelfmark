export type VersionDetectionState =
  | "not_probed"
  | "ok"
  | "exe_missing"
  | "unreadable";

export type SteamCompatView = {
  supportedSteam: string;
  supportedSteamNote: string;
  detectedSteamVersion: string | null;
  steamWithinSupport: boolean | null;
  checkedExePath?: string | null;
  detectionState?: VersionDetectionState | null;
};

function renderTemplate(template: string, params: Record<string, string>): string {
  return template.replace(/\{(\w+)\}/g, (_, k: string) => params[k] ?? `{${k}}`);
}

export function formatCompatSummary(
  c: SteamCompatView,
  labels: {
    supported: string;
    detected: string;
    unknown: string;
    exeMissing: string;
    unreadable: string;
    warnOutside: string;
  },
): { line: string; warn: boolean } {
  let detail: string;
  if (c.detectedSteamVersion) {
    detail = `${labels.detected}: ${c.detectedSteamVersion}`;
  } else {
    const path = c.checkedExePath ?? "";
    switch (c.detectionState) {
      case "exe_missing":
        detail = path ? renderTemplate(labels.exeMissing, { path }) : labels.unknown;
        break;
      case "unreadable":
        detail = path ? renderTemplate(labels.unreadable, { path }) : labels.unknown;
        break;
      default:
        detail = labels.unknown;
    }
  }
  const warn = c.steamWithinSupport === false;
  const warnBit = warn ? ` · ${labels.warnOutside}` : "";
  return {
    line: `${labels.supported}: ${c.supportedSteam} (${c.supportedSteamNote}) · ${detail}${warnBit}`,
    warn,
  };
}
