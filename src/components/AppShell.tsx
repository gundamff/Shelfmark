import type { ReactNode } from "react";
import logo from "../assets/brand/logo-placeholder.svg";
import type { MessageKey } from "../i18n";
import type { SteamCompatView } from "../lib/steamCompat";
import { SteamCompatBar } from "./SteamCompatBar";

export type AppTab = "settings" | "collections";

type Props = {
  tab: AppTab;
  onTab: (tab: AppTab) => void;
  onAbout: () => void;
  sessionChip: string | null;
  compat: SteamCompatView | null;
  t: (key: MessageKey, params?: Record<string, string | number>) => string;
  children: ReactNode;
};

export function AppShell({
  tab,
  onTab,
  onAbout,
  sessionChip,
  compat,
  t,
  children,
}: Props) {
  return (
    <div className="shell">
      <header className="topbar">
        <div className="brand">
          <img src={logo} alt="" width={28} height={28} />
          <span className="brand-name">Shelfmark</span>
          {sessionChip && <span className="session-chip">{sessionChip}</span>}
        </div>
        <nav className="tabs" aria-label="main">
          <button
            type="button"
            className={tab === "settings" ? "tab active" : "tab"}
            onClick={() => onTab("settings")}
          >
            {t("navSettings")}
          </button>
          <button
            type="button"
            className={tab === "collections" ? "tab active" : "tab"}
            onClick={() => onTab("collections")}
          >
            {t("navCollections")}
          </button>
        </nav>
        <button type="button" className="tab ghost" onClick={onAbout}>
          {t("navAbout")}
        </button>
      </header>
      <SteamCompatBar compat={compat} t={t} />
      <div className="shell-main">{children}</div>
    </div>
  );
}
