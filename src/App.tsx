// CubeXCraft Installer (SCRUM-363): одна задача — поставить лаунчер CubeXCraft на iPhone с правом Increased Memory
// Limit и подготовить JIT. Настройки сервера подписи — в «Дополнительно».
import { useCallback, useEffect, useRef, useState } from "react";
import "./App.css";
import "./cxc-theme.css";
import { AppleID } from "./AppleID";
import { Device, DeviceInfo } from "./Device";
import { invoke } from "@tauri-apps/api/core";
import {
  installCubeXCraftOperation,
  Operation,
  OperationState,
  OperationUpdate,
} from "./components/operations";
import { listen } from "@tauri-apps/api/event";
import OperationView from "./components/OperationView";
import { toast } from "sonner";
import { Modal } from "./components/Modal";
import { Settings } from "./pages/Settings";
import { getVersion } from "@tauri-apps/api/app";
import { useTranslation } from "react-i18next";
import { usePlatform } from "./PlatformContext";
import { CubeXCraft } from "./CubeXCraft";
import { HowTo } from "./HowTo";

const Logo = () => (
  <svg width="40" height="45" viewBox="0 0 86 96" fill="none" style={{ flex: "none", filter: "drop-shadow(0 0 12px #ff2a3c88)" }}>
    <path d="M43 2 82 24v48L43 94 4 72V24Z" stroke="#ff2a3c" strokeWidth="4" fill="#ff2a3c14" />
    <path d="M43 26 63 37v22L43 70 23 59V37Z" fill="#ff2a3c" />
    <path d="M43 26 63 37 43 48 23 37Z" fill="#fcee0a" />
  </svg>
);

function App() {
  const { t, i18n } = useTranslation();

  const [operationState, setOperationState] = useState<OperationState | null>(null);
  const [loggedInAs, setLoggedInAs] = useState<string | null>(null);
  const [selectedDevice, setSelectedDevice] = useState<DeviceInfo | null>(null);
  const [advancedOpen, setAdvancedOpen] = useState(false);
  // «Как установить?» — при первом запуске открывается само
  const [howToOpen, setHowToOpen] = useState(() => {
    try {
      return localStorage.getItem("cxh-seen") === null;
    } catch {
      return false;
    }
  });
  const closeHowTo = useCallback(() => {
    setHowToOpen(false);
    try {
      localStorage.setItem("cxh-seen", "1");
    } catch {}
  }, []);
  const [version, setVersion] = useState<string>("");
  const [noKeyringAvailable, setNoKeyringAvailable] = useState<boolean>(false);
  // самообновление (selfupdate.rs): пока качается и ставится новая версия — плашка поверх окна
  const [selfUpdate, setSelfUpdate] = useState<{ state: string; version: string; progress: number } | null>(null);
  const refreshDevicesRef = useRef<(() => void) | null>(null);
  const { platform } = usePlatform();

  const checkKeyring = useCallback(async () => {
    try {
      setNoKeyringAvailable(!(await invoke<boolean>("keyring_available")));
    } catch (e) {
      console.error("Unable to check keyring availability:", e);
      setNoKeyringAvailable(true);
    }
  }, []);

  useEffect(() => {
    checkKeyring();
    getVersion().then(setVersion);
  }, [checkKeyring]);

  useEffect(() => {
    const unlisten = listen<{ state: string; version: string; progress: number }>("self-update", (event) => {
      setSelfUpdate(event.payload.state === "failed" ? null : event.payload);
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const shortcutLabel = useCallback(
    (mac: string, windows: string, linux?: string) => {
      if (platform === "mac") return mac;
      if (platform === "linux") return linux ?? windows;
      return windows;
    },
    [platform],
  );

  const startOperation = useCallback(
    async (operation: Operation, params: { [key: string]: any }): Promise<void> => {
      setOperationState({ current: operation, started: [], failed: [], completed: [], progress: {} });
      return new Promise<void>(async (resolve, reject) => {
        const unlistenFn = await listen<OperationUpdate>("operation_" + operation.id, (event) => {
          setOperationState((old) => {
            if (old == null) return null;
            if (event.payload.updateType === "started") {
              return { ...old, started: [...old.started, event.payload.stepId] };
            } else if (event.payload.updateType === "finished") {
              return { ...old, completed: [...old.completed, event.payload.stepId] };
            } else if (event.payload.updateType === "failed") {
              return { ...old, failed: [...old.failed, { stepId: event.payload.stepId, extraDetails: event.payload.extraDetails }] };
            } else if (event.payload.updateType === "progress") {
              const { stepId, phase, progress } = event.payload;
              return { ...old, progress: { ...old.progress, [stepId]: { phase, progress } } };
            }
            return old;
          });
        });
        try {
          await invoke(operation.id + "_operation", params);
          unlistenFn();
          resolve();
        } catch (e) {
          unlistenFn();
          reject(e);
        }
      });
    },
    [setOperationState],
  );

  const ensuredLoggedIn = useCallback((): boolean => {
    if (loggedInAs) return true;
    toast.error(t("app.must_be_logged_in"));
    return false;
  }, [loggedInAs, t]);

  const ensureSelectedDevice = useCallback((): boolean => {
    if (selectedDevice) return true;
    toast.error(t("app.must_select_device"));
    return false;
  }, [selectedDevice, t]);

  // одна кнопка — свежий лаунчер с сайта + файл сопряжения в StikDebug
  const installCubeXCraft = useCallback(async () => {
    if (!ensuredLoggedIn() || !ensureSelectedDevice()) return;
    await startOperation(installCubeXCraftOperation, { appPath: null });
  }, [ensuredLoggedIn, ensureSelectedDevice, startOperation]);

  useEffect(() => {
    const handler = (event: KeyboardEvent) => {
      if (event.key === undefined) return;
      const primary = platform === "mac" ? event.metaKey : event.ctrlKey;
      if (primary && !event.shiftKey && event.key.toLowerCase() === "r") {
        event.preventDefault();
        refreshDevicesRef.current?.();
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [platform]);

  const lang = i18n.language.startsWith("ru") ? "ru" : "en";
  const kick = (text: string, yellow = false) => (
    <div className={"cxi-kick" + (yellow ? " y" : "")}>
      <span>// {text}</span>
      <span className="ln" />
    </div>
  );

  return (
    <main className="cxi">
      <header className="cxi-head">
        <Logo />
        <div>
          <div className="cxi-brand">
            CUBE<b>X</b>CRAFT
          </div>
          <div className="cxi-sub">// {t("cxc.subtitle")}</div>
        </div>
        <button className="cxi-howto" onClick={() => setHowToOpen(true)}>
          {t("cxc.howto")}
        </button>
        <div className="cxi-seg">
          {(["ru", "en"] as const).map((l) => (
            <span key={l} className={lang === l ? "on" : ""} onClick={() => i18n.changeLanguage(l)}>
              {l.toUpperCase()}
            </span>
          ))}
        </div>
      </header>

      <section>
        {kick(`01 · ${t("cxc.step_apple")}`)}
        <div className={"cxi-step" + (loggedInAs ? " done" : "")}>
          <div className="cxi-body">
            <AppleID loggedInAs={loggedInAs} setLoggedInAs={setLoggedInAs} noKeyringAvailable={noKeyringAvailable} />
          </div>
        </div>
      </section>

      <section>
        {kick(`02 · ${t("cxc.step_device")}`)}
        <div className={"cxi-step" + (selectedDevice ? " done" : "")}>
          <div className="cxi-body">
            <Device
              selectedDevice={selectedDevice}
              setSelectedDevice={setSelectedDevice}
              registerRefresh={(fn) => {
                refreshDevicesRef.current = fn ?? null;
              }}
            />
          </div>
        </div>
      </section>

      <section>
        {kick(`03 · ${t("cxc.step_install")}`, true)}
        <div className="cxi-step">
          <CubeXCraft selectedDevice={selectedDevice} onInstall={installCubeXCraft} />
        </div>
      </section>

      <footer className="cxi-foot">
        <span>
          {t("version")} {version}
        </span>
        <span className="link" onClick={() => setAdvancedOpen(true)}>
          {t("cxc.advanced")}
        </span>
      </footer>

      <HowTo isOpen={howToOpen} close={closeHowTo} lang={lang} platform={platform} />
      {operationState && <OperationView operationState={operationState} closeMenu={() => setOperationState(null)} />}
      <Modal sizeFit isOpen={selfUpdate !== null} zIndex={3000}>
        <h2>{t("update.self_title")}</h2>
        <p className="cxc-phase">
          {selfUpdate?.state === "restarting"
            ? t("update.self_restarting")
            : t("update.self_downloading", { version: selfUpdate?.version, pct: selfUpdate?.progress })}
        </p>
      </Modal>
      <Modal isOpen={advancedOpen} close={() => setAdvancedOpen(false)}>
        <h2>{t("cxc.advanced")}</h2>
        <p className="cxc-hint">{t("cxc.licenses")}</p>
        <Settings
          ensureSelectedDevice={ensureSelectedDevice}
          setSelectedDevice={setSelectedDevice}
          platform={platform}
          shortcutLabel={shortcutLabel}
          checkKeyring={checkKeyring}
        />
      </Modal>
    </main>
  );
}

export default App;
