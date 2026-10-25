// CubeXCraft (windows-claude, SCRUM-363): «одна кнопка» — установить лаунчер CubeXCraft и подготовить JIT.
// Что стоит на iPhone, проверяет src-tauri/src/cubexcraft.rs; чего нет — QR-код на App Store.
import "./CubeXCraft.css";
import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { DeviceInfo } from "./Device";

type CubeXCraftStatus = {
  cubexcraft: boolean;
  stikdebug: string | null;
  localdevvpn: boolean;
};

// автопродление (SCRUM-368): когда iPhone ставили и когда установщик продлит сам
type RenewStatus = {
  udid: string;
  name: string;
  installedAt: number;
  renewAt: number;
  lastError: string | null;
};

const APP_STORE = {
  stikdebug: "https://apps.apple.com/app/id6744045754",
  localdevvpn: "https://apps.apple.com/app/id6755608044",
} as const;

export const CubeXCraft = ({
  selectedDevice,
  onInstall,
}: {
  selectedDevice: DeviceInfo | null;
  onInstall: () => Promise<void>;
}) => {
  const { t } = useTranslation();
  const [status, setStatus] = useState<CubeXCraftStatus | null>(null);
  const [checking, setChecking] = useState(false);
  const [qr, setQr] = useState<Record<string, string>>({});
  const [renew, setRenew] = useState<RenewStatus[]>([]);
  const [renewing, setRenewing] = useState(false);

  const loadRenew = useCallback(async () => {
    try {
      setRenew(await invoke<RenewStatus[]>("cubexcraft_renew_status"));
    } catch (e) {
      console.error("Renew status failed", e);
    }
  }, []);

  useEffect(() => {
    loadRenew();
  }, [loadRenew]);

  const check = useCallback(async () => {
    loadRenew();
    if (!selectedDevice) {
      setStatus(null);
      return;
    }
    setChecking(true);
    try {
      setStatus(await invoke<CubeXCraftStatus>("cubexcraft_status"));
    } catch (e) {
      console.error("CubeXCraft status failed", e);
      setStatus(null);
    } finally {
      setChecking(false);
    }
  }, [selectedDevice]);

  useEffect(() => {
    check();
  }, [check]);

  useEffect(() => {
    (async () => {
      const codes: Record<string, string> = {};
      for (const [key, url] of Object.entries(APP_STORE)) {
        try {
          codes[key] = await invoke<string>("cubexcraft_qr", { text: url });
        } catch (e) {
          console.error("QR failed", e);
        }
      }
      setQr(codes);
    })();
  }, []);

  const row = (label: string, ok: boolean | undefined) => (
    <div className="cxc-row">
      <span>{label}</span>
      <span className={ok ? "cxc-ok" : "cxc-missing"}>
        {ok === undefined ? "—" : ok ? t("cubexcraft.installed") : t("cubexcraft.not_installed")}
      </span>
    </div>
  );

  const missing = status
    ? (["localdevvpn"] as ("stikdebug" | "localdevvpn")[]).filter((key) =>
        key === "stikdebug" ? !status.stikdebug : !status.localdevvpn,
      )
    : [];

  return (
    <div className="cxc">
      <div className="action-row single-row">
        <button
          className="cxc-install"
          onClick={async () => {
            try {
              await onInstall();
            } catch (e) {
              console.error(e);
            }
            check();
          }}
        >
          {t("cubexcraft.install")}
        </button>
        <button onClick={check} disabled={!selectedDevice || checking}>
          {checking ? t("cubexcraft.checking") : t("cubexcraft.check")}
        </button>
      </div>
      {!selectedDevice ? (
        <p className="cxc-hint">{t("cubexcraft.select_device_first")}</p>
      ) : (
        <div className="cxc-status">
          {row(t("cubexcraft.status_cubexcraft"), status?.cubexcraft)}
          {row(t("cubexcraft.status_stikdebug"), status ? !!status.stikdebug : undefined)}
          {row(t("cubexcraft.status_localdevvpn"), status?.localdevvpn)}
        </div>
      )}
      {renew.length > 0 && (
        <div className="cxc-renew">
          {renew.map((r) => (
            <p className="cxc-hint" key={r.udid}>
              {t("cubexcraft.renew_line", {
                name: r.name,
                date: new Date(r.renewAt * 1000).toLocaleDateString(),
              })}
              {r.lastError ? " " + t("cubexcraft.renew_failed") : ""}
            </p>
          ))}
          <p className="cxc-hint">{t("cubexcraft.renew_wifi_hint")}</p>
          <button
            disabled={renewing}
            onClick={async () => {
              setRenewing(true);
              try {
                await invoke("cubexcraft_renew_now");
              } catch (e) {
                console.error(e);
              }
              setRenewing(false);
              loadRenew();
            }}
          >
            {renewing ? t("cubexcraft.renewing") : t("cubexcraft.renew_now")}
          </button>
        </div>
      )}
      {missing.length > 0 && (
        <div className="cxc-store">
          <p className="cxc-hint">{t("cubexcraft.get_from_app_store")}</p>
          <div className="cxc-qr-list">
            {missing.map((key) => (
              <div className="cxc-qr" key={key}>
                <div
                  className="cxc-qr-image"
                  dangerouslySetInnerHTML={{ __html: qr[key] ?? "" }}
                />
                <span>{key === "stikdebug" ? "StikDebug" : "LocalDevVPN"}</span>
                <button onClick={() => openUrl(APP_STORE[key])}>
                  {t("cubexcraft.open_app_store")}
                </button>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};

// Шаг «StikDebug и LocalDevVPN из App Store» в окне установки: пока установщик ждёт, показываем QR-коды
// того, чего ещё нет на iPhone; статус обновляется сам каждые 3 с
export const CubeXCraftAppsWait = () => {
  const { t } = useTranslation();
  const [status, setStatus] = useState<CubeXCraftStatus | null>(null);
  const [qr, setQr] = useState<Record<string, string>>({});

  useEffect(() => {
    let alive = true;
    const poll = async () => {
      try {
        const s = await invoke<CubeXCraftStatus>("cubexcraft_status");
        if (alive) setStatus(s);
      } catch (e) {
        console.error("CubeXCraft status failed", e);
      }
    };
    poll();
    const timer = setInterval(poll, 3000);
    (async () => {
      const codes: Record<string, string> = {};
      for (const [key, url] of Object.entries(APP_STORE)) {
        try {
          codes[key] = await invoke<string>("cubexcraft_qr", { text: url });
        } catch (e) {
          console.error("QR failed", e);
        }
      }
      if (alive) setQr(codes);
    })();
    return () => {
      alive = false;
      clearInterval(timer);
    };
  }, []);

  const missing = (["localdevvpn"] as ("stikdebug" | "localdevvpn")[]).filter((key) =>
    !status ? true : key === "stikdebug" ? !status.stikdebug : !status.localdevvpn,
  );
  if (missing.length === 0) return null;

  return (
    <div className="cxc-store cxc-wait">
      <p className="cxc-hint">{t("cubexcraft.wait_apps_hint")}</p>
      <div className="cxc-qr-list">
        {missing.map((key) => (
          <div className="cxc-qr" key={key}>
            <div className="cxc-qr-image" dangerouslySetInnerHTML={{ __html: qr[key] ?? "" }} />
            <span>{key === "stikdebug" ? "StikDebug" : "LocalDevVPN"}</span>
          </div>
        ))}
      </div>
    </div>
  );
};
