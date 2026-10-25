// CubeXCraft (windows-claude, 25.10): самообновление установщика — как у лаунчера (launcher-own selfupdate.rs).
// На сайте /cxl/launcher/installer-<платформа>.json {version,url,sha256,size}. При запуске и раз в 6 часов в трее
// установщик сверяет версию, качает новую, ставит её на своё место и перезапускается. Посреди установки или
// автопродления не обновляется — ждёт. Windows: exe (запущенный exe переименовать можно — в .old, удалить нельзя);
// Mac: архив всего .app. Хвосты (.old) удаляются при следующем запуске.
use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicU8, AtomicUsize, Ordering},
    time::Duration,
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Emitter, Manager};
use tracing::{info, warn};

const SITE: &str = "https://www.cubexcraft.site/cxl/launcher";
const CHECK_EVERY: Duration = Duration::from_secs(6 * 3600);
const AFTER_UPDATE_ARG: &str = "--after-update";

// идёт установка или автопродление — обновление подождёт
static BUSY: AtomicUsize = AtomicUsize::new(0);

pub struct BusyGuard;

impl BusyGuard {
    pub fn new() -> Self {
        BUSY.fetch_add(1, Ordering::SeqCst);
        BusyGuard
    }
}

impl Drop for BusyGuard {
    fn drop(&mut self) {
        BUSY.fetch_sub(1, Ordering::SeqCst);
    }
}

#[derive(Deserialize)]
struct Release {
    version: String,
    url: String,
    sha256: String,
    size: u64,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct UpdateEvent {
    // downloading | restarting | failed
    state: &'static str,
    version: String,
    progress: u8,
}

fn platform_key() -> &'static str {
    if cfg!(target_os = "macos") {
        "installer-macos-app"
    } else {
        "installer-windows-x64"
    }
}

fn newer(remote: &str, local: &str) -> bool {
    let parse = |s: &str| s.split('.').map(|x| x.parse::<u64>().unwrap_or(0)).collect::<Vec<_>>();
    parse(remote) > parse(local)
}

/// до запуска Tauri: после обновления дождаться выхода старой копии и убрать её хвосты
pub fn on_startup() {
    if std::env::args().any(|a| a == AFTER_UPDATE_ARG) {
        std::thread::sleep(Duration::from_millis(1500));
    }
    let Ok(me) = std::env::current_exe() else { return };
    let _ = std::fs::remove_file(format!("{}.old", me.to_string_lossy()));
    #[cfg(target_os = "macos")]
    if let Some(bundle) = bundle_of(&me) {
        let _ = std::fs::remove_dir_all(format!("{}.old", bundle.to_string_lossy()));
    }
}

pub fn start(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        // окно успевает подписаться на события
        tokio::time::sleep(Duration::from_secs(2)).await;
        loop {
            run_once(&app).await;
            tokio::time::sleep(CHECK_EVERY).await;
        }
    });
}

async fn run_once(app: &AppHandle) {
    let http = match reqwest::Client::builder()
        .user_agent(concat!("CubeXCraft-Installer/", env!("CARGO_PKG_VERSION")))
        .build()
    {
        Ok(http) => http,
        Err(e) => return warn!("selfupdate: http client: {e}"),
    };
    let Some(release) = check(&http).await else { return };
    info!("selfupdate: {} -> {}", env!("CARGO_PKG_VERSION"), release.version);
    let visible = app
        .get_webview_window("main")
        .and_then(|w| w.is_visible().ok())
        .unwrap_or(false);
    let emit = |state: &'static str, progress: u8| {
        let _ = app.emit(
            "self-update",
            UpdateEvent { state, version: release.version.clone(), progress },
        );
    };
    while BUSY.load(Ordering::SeqCst) > 0 {
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
    emit("downloading", 0);
    let last = AtomicU8::new(0);
    match apply(&http, &release, visible, &|done, total| {
        let pct = (done * 100 / total.max(1)).min(100) as u8;
        if last.swap(pct, Ordering::Relaxed) != pct {
            emit("downloading", pct);
        }
    })
    .await
    {
        Ok(()) => {
            info!("selfupdate: installed {}, restarting", release.version);
            emit("restarting", 100);
            tokio::time::sleep(Duration::from_millis(300)).await;
            app.exit(0);
        }
        Err(e) => {
            warn!("selfupdate: failed: {e}");
            emit("failed", 0);
        }
    }
}

async fn check(http: &reqwest::Client) -> Option<Release> {
    let response = http
        .get(format!("{SITE}/{}.json", platform_key()))
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .ok()?;
    if !response.status().is_success() {
        return None;
    }
    let release: Release = serde_json::from_str(&response.text().await.ok()?).ok()?;
    newer(&release.version, env!("CARGO_PKG_VERSION")).then_some(release)
}

/// скачать, сверить сумму, поставить на место текущего и запустить новую копию
async fn apply(
    http: &reqwest::Client,
    release: &Release,
    visible: bool,
    emit: &(dyn Fn(u64, u64) + Send + Sync),
) -> Result<(), String> {
    let me = std::env::current_exe().map_err(|e| e.to_string())?;
    let tmp = if cfg!(target_os = "macos") {
        std::env::temp_dir().join(format!("cubexcraft-installer-update-{}.tar.gz", std::process::id()))
    } else {
        PathBuf::from(format!("{}.download", me.to_string_lossy()))
    };
    let mut response = http
        .get(&release.url)
        .send()
        .await
        .map_err(|e| e.without_url().to_string())?;
    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status().as_u16()));
    }
    let mut bytes = Vec::with_capacity(release.size as usize);
    let mut hasher = Sha256::new();
    while let Some(chunk) = response.chunk().await.map_err(|e| e.without_url().to_string())? {
        hasher.update(&chunk);
        bytes.extend_from_slice(&chunk);
        emit(bytes.len() as u64, release.size);
    }
    if format!("{:x}", hasher.finalize()) != release.sha256.to_lowercase() {
        return Err("update checksum mismatch".into());
    }
    std::fs::write(&tmp, &bytes).map_err(|e| e.to_string())?;

    let mut args = vec![AFTER_UPDATE_ARG];
    if !visible {
        args.push("--hidden");
    }

    replace_and_restart(&me, &tmp, &args)
}

#[cfg(target_os = "macos")]
fn replace_and_restart(me: &Path, tmp: &Path, args: &[&str]) -> Result<(), String> {
    let result = apply_mac_bundle(me, tmp, args);
    let _ = std::fs::remove_file(tmp);
    result
}

#[cfg(not(target_os = "macos"))]
fn replace_and_restart(me: &Path, tmp: &Path, args: &[&str]) -> Result<(), String> {
    let old = PathBuf::from(format!("{}.old", me.to_string_lossy()));
    let _ = std::fs::remove_file(&old);
    std::fs::rename(me, &old).map_err(|e| e.to_string())?;
    if let Err(e) = std::fs::rename(tmp, me) {
        let _ = std::fs::rename(&old, me);
        return Err(e.to_string());
    }
    let mut cmd = std::process::Command::new(me);
    cmd.args(args);
    if let Some(dir) = me.parent() {
        cmd.current_dir(dir);
    }
    cmd.spawn().map_err(|e| e.to_string())?;
    Ok(())
}

/// .app, внутри которого мы запущены (…/CubeXCraft Installer.app/Contents/MacOS/cubexcraft-installer)
#[cfg(target_os = "macos")]
fn bundle_of(exe: &Path) -> Option<PathBuf> {
    exe.ancestors()
        .find(|p| p.extension().is_some_and(|e| e == "app"))
        .map(Path::to_path_buf)
}

/// macOS: распаковать архив .app рядом с текущим (тот же том), старый -> .app.old, новый на его место,
/// запуск нового через LaunchServices
#[cfg(target_os = "macos")]
fn apply_mac_bundle(me: &Path, archive: &Path, args: &[&str]) -> Result<(), String> {
    let bundle = bundle_of(me).ok_or("installer is not running from an .app")?;
    let parent = bundle.parent().ok_or("no app folder")?.to_path_buf();
    let staging = parent.join(format!(".cubexcraft-installer-update-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&staging);
    std::fs::create_dir_all(&staging).map_err(|e| format!("{}: {e}", parent.display()))?;
    let file = std::fs::File::open(archive).map_err(|e| e.to_string())?;
    tar::Archive::new(flate2::read::GzDecoder::new(file))
        .unpack(&staging)
        .map_err(|e| e.to_string())?;
    let new_app = std::fs::read_dir(&staging)
        .map_err(|e| e.to_string())?
        .flatten()
        .map(|e| e.path())
        .find(|p| p.extension().is_some_and(|e| e == "app"))
        .ok_or("no .app in the update archive")?;
    let old = PathBuf::from(format!("{}.old", bundle.to_string_lossy()));
    let _ = std::fs::remove_dir_all(&old);
    std::fs::rename(&bundle, &old).map_err(|e| e.to_string())?;
    if let Err(e) = std::fs::rename(&new_app, &bundle) {
        let _ = std::fs::rename(&old, &bundle);
        let _ = std::fs::remove_dir_all(&staging);
        return Err(e.to_string());
    }
    let _ = std::fs::remove_dir_all(&staging);
    std::process::Command::new("open")
        .arg("-n")
        .arg(&bundle)
        .arg("--args")
        .args(args)
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}
