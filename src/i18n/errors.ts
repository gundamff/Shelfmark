import { t } from "./translate";
import type { Locale, MessageKey } from "./types";

const RULES: { match: RegExp; key: MessageKey }[] = [
  { match: /no session open/i, key: "errNoSession" },
  {
    match: /close steam before export|steam is running|cannot export while steam/i,
    key: "errSteamRunning",
  },
  { match: /catalog not found/i, key: "errCatalogMissing" },
  { match: /run autocat first/i, key: "errAutocatFirst" },
  { match: /windows-only/i, key: "errWindowsOnly" },
];

export function localizeError(locale: Locale, raw: string): string {
  const text = raw.replace(/^Error:\s*/i, "").trim();
  for (const rule of RULES) {
    if (rule.match.test(text)) {
      return t(locale, rule.key);
    }
  }
  return text;
}
