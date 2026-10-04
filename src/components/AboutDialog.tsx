import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import logo from "../assets/brand/shelfmark-logo.jpg";
import { GITHUB_REPO_URL } from "../config/brand";
import type { MessageKey } from "../i18n";
import type { SteamCompatView } from "../lib/steamCompat";
import { SteamCompatBar } from "./SteamCompatBar";

type Props = {
  open: boolean;
  onClose: () => void;
  compat: SteamCompatView | null;
  t: (key: MessageKey, params?: Record<string, string | number>) => string;
};

export function AboutDialog({ open, onClose, compat, t }: Props) {
  const [version, setVersion] = useState<string | null>(null);

  useEffect(() => {
    if (!open) return;
    void invoke<string>("app_version")
      .then(setVersion)
      .catch(() => setVersion(null));
  }, [open]);

  if (!open) return null;

  return (
    <div
      className="modal-mask"
      role="presentation"
      onClick={onClose}
      onKeyDown={(e) => {
        if (e.key === "Escape") onClose();
      }}
    >
      <div
        className="modal-card"
        role="dialog"
        aria-labelledby="about-title"
        onClick={(e) => e.stopPropagation()}
      >
        <img className="about-logo" src={logo} alt="" width={72} height={72} />
        <h2 id="about-title">{t("aboutTitle")}</h2>
        <p className="muted">{t("tagline")}</p>
        {version && (
          <p className="muted">{t("aboutVersion", { version })}</p>
        )}
        <SteamCompatBar compat={compat} t={t} />
        <p className="about-disclaimer">{t("aboutDisclaimer")}</p>
        <div className="row" style={{ justifyContent: "center" }}>
          {GITHUB_REPO_URL ? (
            <a
              className="button-link"
              href={GITHUB_REPO_URL}
              target="_blank"
              rel="noreferrer"
            >
              {t("aboutGithub")}
            </a>
          ) : null}
          <button type="button" onClick={onClose}>
            {t("aboutClose")}
          </button>
        </div>
      </div>
    </div>
  );
}
