import { en } from "./en";
import { zh } from "./zh";
import type { Locale, LocalePreference, MessageKey, Messages } from "./types";

const DICTS: Record<Locale, Messages> = { en, zh };

export function resolveLocale(
  preference: LocalePreference | null | undefined,
  systemLanguage: string,
): Locale {
  if (preference === "en" || preference === "zh") {
    return preference;
  }
  const lang = systemLanguage.toLowerCase();
  return lang.startsWith("zh") ? "zh" : "en";
}

export function t(
  locale: Locale,
  key: MessageKey,
  params?: Record<string, string | number>,
): string {
  let s = DICTS[locale][key] ?? DICTS.en[key] ?? key;
  if (params) {
    for (const [k, v] of Object.entries(params)) {
      s = s.split(`{${k}}`).join(String(v));
    }
  }
  return s;
}

export function parseLocalePreference(
  value: string | null | undefined,
): LocalePreference {
  if (value === "en" || value === "zh" || value === "system") {
    return value;
  }
  return "system";
}
