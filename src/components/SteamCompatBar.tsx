import { formatCompatSummary, type SteamCompatView } from "../lib/steamCompat";
import type { MessageKey } from "../i18n";

type Props = {
  compat: SteamCompatView | null;
  t: (key: MessageKey, params?: Record<string, string | number>) => string;
};

export function SteamCompatBar({ compat, t }: Props) {
  const view: SteamCompatView = compat ?? {
    supportedSteam: "",
    supportedSteamNote: "",
    detectedSteamVersion: null,
    steamWithinSupport: null,
  };

  // When detect not run yet, still show supported constants from a bootstrap call.
  const { line, warn } = formatCompatSummary(
    {
      supportedSteam: view.supportedSteam || "…",
      supportedSteamNote: view.supportedSteamNote || "…",
      detectedSteamVersion: view.detectedSteamVersion,
      steamWithinSupport: view.steamWithinSupport,
    },
    {
      supported: t("compatSupported"),
      detected: t("compatDetected"),
      unknown: t("compatUnknown"),
      warnOutside: t("compatWarnOutside"),
    },
  );

  return (
    <div className={`compat-bar${warn ? " warn" : ""}`} role="status">
      {line}
    </div>
  );
}
