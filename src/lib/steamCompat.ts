export type SteamCompatView = {
  supportedSteam: string;
  supportedSteamNote: string;
  detectedSteamVersion: string | null;
  steamWithinSupport: boolean | null;
};

export function formatCompatSummary(
  c: SteamCompatView,
  labels: {
    supported: string;
    detected: string;
    unknown: string;
    warnOutside: string;
  },
): { line: string; warn: boolean } {
  const detected = c.detectedSteamVersion
    ? `${labels.detected}: ${c.detectedSteamVersion}`
    : labels.unknown;
  const warn = c.steamWithinSupport === false;
  const warnBit = warn ? ` · ${labels.warnOutside}` : "";
  return {
    line: `${labels.supported}: ${c.supportedSteam} (${c.supportedSteamNote}) · ${detected}${warnBit}`,
    warn,
  };
}
