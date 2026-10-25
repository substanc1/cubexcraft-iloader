// CubeXCraft (windows-claude, SCRUM-368): автопродление. Бесплатный Apple ID подписывает CubeXCraft на 7 дней —
// установщик живёт в трее, стартует со входом в систему и раз в 20 минут смотрит, не пора ли: прошло 5 дней или
// на сайте новая версия. iPhone виден (Wi-Fi или кабель) — переподписывает сам и сообщает уведомлением.
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_store::StoreExt;
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};

use crate::{
    account::login_stored,
    cubexcraft::{fetch_ipa, fetch_stikdebug},
    device::{DeviceInfo, get_provider, get_usbmuxd, is_wifi_only, list_devices},
    error::AppError,
    fastinstall,
    pairing::pairing_file,
    sideload::{SideloaderGuard, SideloaderMutex},
};

// продлеваем за 2 дня до конца подписи — запас, если компьютер был выключен
const RENEW_AFTER_SECS: i64 = 5 * 24 * 3600;
const CHECK_EVERY: Duration = Duration::from_secs(20 * 60);
// об одной и той же неудаче напоминаем не чаще раза в 12 часов
const ERROR_NOTICE_EVERY_SECS: i64 = 12 * 3600;
const STATE_FILE: &str = "cubexcraft-renew.json";
const DEFAULT_ANISETTE: &str = "mac-ani.jkcoxson.com";

static RUNNING: AtomicBool = AtomicBool::new(false);

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RenewEntry {
    pub name: String,
    pub email: String,
    pub installed_at: i64,
    // версия с сайта; пусто — ставили своим файлом
    pub version: String,
    // свой IPA: продлеваем им же, пока файл на месте
    pub file: Option<String>,
    // StikDebug ставили мы (с GitHub) — продлеваем вместе с CubeXCraft
    #[serde(default)]
    pub stikdebug: bool,
    pub last_error: Option<String>,
    pub last_error_at: Option<i64>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct RenewState {
    pub devices: HashMap<String, RenewEntry>,
}

fn state_path(app: &AppHandle) -> Option<PathBuf> {
    app.path().app_data_dir().ok().map(|dir| dir.join(STATE_FILE))
}

fn load(app: &AppHandle) -> RenewState {
    state_path(app)
        .and_then(|path| std::fs::read(path).ok())
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default()
}

fn save(app: &AppHandle, state: &RenewState) {
    let Some(path) = state_path(app) else { return };
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).ok();
    }
    match serde_json::to_vec_pretty(state) {
        Ok(bytes) => {
            if let Err(e) = std::fs::write(&path, bytes) {
                warn!("autorenew: failed to save state: {e}");
            }
        }
        Err(e) => warn!("autorenew: failed to serialize state: {e}"),
    }
}

fn now() -> i64 {
    chrono::Utc::now().timestamp()
}

fn notify(app: &AppHandle, body: &str) {
    if let Err(e) = app.notification().builder().title("CubeXCraft").body(body).show() {
        warn!("autorenew: notification failed: {e}");
    }
}

pub fn show_main(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

// после установки из окна: запомнить iPhone, чтобы продлевать его самим
pub fn record_install(
    app: &AppHandle,
    device: &DeviceInfo,
    email: &str,
    version: &str,
    file: Option<String>,
    stikdebug: bool,
) {
    let mut state = load(app);
    let stikdebug = stikdebug || state.devices.get(&device.udid).is_some_and(|e| e.stikdebug);
    state.devices.insert(
        device.udid.clone(),
        RenewEntry {
            name: device.name.clone(),
            email: email.to_string(),
            installed_at: now(),
            version: version.to_string(),
            file,
            stikdebug,
            last_error: None,
            last_error_at: None,
        },
    );
    save(app, &state);
}

pub fn stikdebug_is_ours(app: &AppHandle, udid: &str) -> bool {
    load(app).devices.get(udid).is_some_and(|e| e.stikdebug)
}

pub fn start(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_secs(90)).await;
        loop {
            run_once(&app, false).await;
            tokio::time::sleep(CHECK_EVERY).await;
        }
    });
}

async fn latest_version() -> Option<String> {
    let text = reqwest::get(crate::cubexcraft::IOS_MANIFEST_URL)
        .await
        .ok()?
        .error_for_status()
        .ok()?
        .text()
        .await
        .ok()?;
    let value: serde_json::Value = serde_json::from_str(&text).ok()?;
    value.get("version")?.as_str().map(str::to_string)
}

// force — «Продлить сейчас» из меню трея: не ждём 5 дней
pub async fn run_once(app: &AppHandle, force: bool) {
    if RUNNING.swap(true, Ordering::SeqCst) {
        return;
    }
    let _busy = crate::selfupdate::BusyGuard::new();
    let mut state = load(app);
    if !state.devices.is_empty() {
        let latest = latest_version().await;
        let visible: Vec<String> = match get_usbmuxd().await {
            Ok(mut usbmuxd) => usbmuxd
                .get_devices()
                .await
                .map(|devs| devs.into_iter().map(|d| d.udid).collect())
                .unwrap_or_default(),
            Err(_) => vec![],
        };
        let mut changed = false;
        for (udid, entry) in state.devices.clone() {
            let outdated = entry.file.is_none()
                && latest.as_ref().is_some_and(|version| *version != entry.version);
            let due = force || now() - entry.installed_at >= RENEW_AFTER_SECS || outdated;
            if !due {
                continue;
            }
            if !visible.contains(&udid) {
                info!("autorenew: {} is not visible over Wi-Fi or USB", entry.name);
                if force {
                    notify(app, &format!(
                        "{} не виден. Подключите iPhone кабелем или к той же Wi-Fi, что и компьютер.",
                        entry.name
                    ));
                }
                continue;
            }
            changed = true;
            let slot = state.devices.get_mut(&udid).expect("entry exists");
            match renew(app, &udid, &entry).await {
                Ok(version) => {
                    info!("autorenew: renewed CubeXCraft on {}", entry.name);
                    slot.installed_at = now();
                    if slot.file.is_none() {
                        slot.version = version;
                    }
                    slot.last_error = None;
                    slot.last_error_at = None;
                    notify(app, &format!("CubeXCraft на {} продлён ещё на 7 дней.", entry.name));
                }
                Err(e) => {
                    let message = format!("{e:?}");
                    warn!("autorenew: renewing {} failed: {message}", entry.name);
                    let remind = slot
                        .last_error_at
                        .is_none_or(|at| now() - at >= ERROR_NOTICE_EVERY_SECS);
                    if force || remind {
                        notify(app, &format!(
                            "Не удалось продлить CubeXCraft на {}. Откройте установщик — там будет ошибка.",
                            entry.name
                        ));
                        slot.last_error_at = Some(now());
                    }
                    slot.last_error = Some(message);
                }
            }
        }
        if changed {
            save(app, &state);
        }
    } else if force {
        notify(app, "Сначала установите CubeXCraft кнопкой «Установить CubeXCraft».");
    }
    RUNNING.store(false, Ordering::SeqCst);
}

// переподписать CubeXCraft на iPhone; возвращает версию, которую поставили
async fn renew(app: &AppHandle, udid: &str, entry: &RenewEntry) -> Result<String, AppError> {
    let info = list_devices()
        .await?
        .into_iter()
        .filter_map(Result::ok)
        .find(|d| d.udid == udid)
        .ok_or(AppError::NoDeviceSelected)?;

    let mut usbmuxd = get_usbmuxd().await?;
    let pairing = pairing_file(app, &info, &mut usbmuxd, CancellationToken::new()).await?;

    // вход тем же Apple ID; если попросит код — окно покажется само (там окно ввода кода)
    let sideloader_state = app.state::<SideloaderMutex>();
    let logged_in = sideloader_state
        .lock()
        .unwrap()
        .as_ref()
        .is_some_and(|s| s.get_email().eq_ignore_ascii_case(&entry.email));
    if !logged_in {
        let window = app
            .get_webview_window("main")
            .ok_or_else(|| AppError::Misc("Main window is missing".into()))?;
        let anisette = app
            .store("preferences.json")
            .ok()
            .and_then(|store| store.get("anisetteServer3-1"))
            .and_then(|value| value.as_str().map(str::to_string))
            .unwrap_or_else(|| DEFAULT_ANISETTE.to_string());
        login_stored(
            app.clone(),
            window.as_ref().window(),
            entry.email.clone(),
            anisette,
            sideloader_state.clone(),
        )
        .await?;
    }

    let (path, version) = match entry.file.as_ref().filter(|f| std::path::Path::new(f).exists()) {
        Some(file) => (PathBuf::from(file), entry.version.clone()),
        None => fetch_ipa(app).await?,
    };

    // быстрая установка: одним сжатым IPA, по Wi-Fi это втрое быстрее
    let provider = get_provider(&info).await?;
    let wifi = is_wifi_only(udid).await;
    let mut sideloader = SideloaderGuard::take(&sideloader_state)?;
    fastinstall::install(&mut sideloader, &provider, path, &pairing, wifi).await?;
    if entry.stikdebug {
        let (stik_path, _) = fetch_stikdebug(app).await?;
        fastinstall::install(&mut sideloader, &provider, stik_path, &pairing, wifi).await?;
    }
    Ok(version)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenewStatusEntry {
    pub udid: String,
    pub name: String,
    pub installed_at: i64,
    pub renew_at: i64,
    pub last_error: Option<String>,
}

#[tauri::command]
pub fn cubexcraft_renew_status(app: AppHandle) -> Vec<RenewStatusEntry> {
    load(&app)
        .devices
        .into_iter()
        .map(|(udid, e)| RenewStatusEntry {
            udid,
            name: e.name,
            installed_at: e.installed_at,
            renew_at: e.installed_at + RENEW_AFTER_SECS,
            last_error: e.last_error,
        })
        .collect()
}

#[tauri::command]
pub async fn cubexcraft_renew_now(app: AppHandle) {
    run_once(&app, true).await;
}
