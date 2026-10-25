// CubeXCraft (windows-claude, 25.10): на Windows с iPhone говорит служба Apple Mobile Device (usbmuxd, 127.0.0.1:27015).
// Владелец: «чтоб проверяло, есть ли айтюнс, и если нет — автоматически загружало, а не писало «перейдите и скачайте»;
// через маркет не надо — не у всех он есть». Службы нет — качаем установщик iTunes с apple.com, достаём из его CAB только
// AppleMobileDeviceSupport64.msi (expand.exe, сам установщик Apple не запускаем), проверяем подпись Apple и ставим
// тихо — Windows один раз спрашивает разрешение администратора. Не вышло — «Устройства Apple» через winget (Microsoft
// Store, без прав администратора). Окно показывает, что идёт.
use std::sync::Mutex;

use serde::Serialize;
use tauri::{AppHandle, Emitter};

#[derive(Serialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub struct AppleServiceState {
    // downloading | installing | ready | restart | failed
    state: &'static str,
    progress: u8,
}

// None — служба есть или проверка ещё не шла
static STATE: Mutex<Option<AppleServiceState>> = Mutex::new(None);

#[allow(dead_code)]
fn set_state(app: &AppHandle, state: &'static str, progress: u8) {
    let value = AppleServiceState { state, progress };
    *STATE.lock().unwrap() = Some(value);
    let _ = app.emit("apple-service", value);
}

#[tauri::command]
pub fn apple_service_state() -> Option<AppleServiceState> {
    *STATE.lock().unwrap()
}

#[cfg(windows)]
mod imp {
    use std::{
        io::{Read, Seek, SeekFrom, Write},
        os::windows::process::CommandExt,
        path::{Path, PathBuf},
        sync::atomic::{AtomicBool, Ordering},
        time::Duration,
    };

    use tauri::AppHandle;
    use tracing::{info, warn};

    use super::set_state;

    const ITUNES_URL: &str = "https://www.apple.com/itunes/download/win64";
    const MSI_NAME: &str = "AppleMobileDeviceSupport64.msi";
    const APPLE_DEVICES_STORE_ID: &str = "9NP83LWLPZ9K";
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    static RUNNING: AtomicBool = AtomicBool::new(false);

    pub fn service_up() -> bool {
        std::net::TcpStream::connect_timeout(
            &std::net::SocketAddr::from(([127, 0, 0, 1], 27015)),
            Duration::from_millis(800),
        )
        .is_ok()
    }

    async fn service_up_async() -> bool {
        tauri::async_runtime::spawn_blocking(service_up).await.unwrap_or(false)
    }

    pub async fn ensure(app: &AppHandle) {
        if service_up_async().await || RUNNING.swap(true, Ordering::SeqCst) {
            return;
        }
        let dir = std::env::temp_dir().join("cubexcraft-apple-mobile-device");
        let installed = match from_apple(app, &dir).await {
            Ok(()) => true,
            Err(e) => {
                warn!("applesvc: Apple Mobile Device Support from apple.com failed: {e}");
                set_state(app, "installing", 0);
                tauri::async_runtime::spawn_blocking(winget_apple_devices).await.unwrap_or(false)
            }
        };
        let _ = std::fs::remove_dir_all(&dir);
        // служба поднимается не сразу после установки
        let mut up = false;
        for _ in 0..30 {
            if service_up_async().await {
                up = true;
                break;
            }
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
        let state = match (up, installed) {
            (true, _) => "ready",
            (false, true) => "restart",
            (false, false) => "failed",
        };
        info!("applesvc: {state}");
        set_state(app, state, 0);
        RUNNING.store(false, Ordering::SeqCst);
    }

    async fn from_apple(app: &AppHandle, dir: &Path) -> Result<(), String> {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        let setup = dir.join("iTunes64Setup.exe");
        download(app, &setup).await?;
        let msi = tauri::async_runtime::spawn_blocking({
            let dir = dir.to_path_buf();
            move || extract_msi(&setup, &dir)
        })
        .await
        .map_err(|e| e.to_string())??;
        set_state(app, "installing", 0);
        tauri::async_runtime::spawn_blocking(move || {
            check_apple_signature(&msi)?;
            install_msi_elevated(&msi)
        })
        .await
        .map_err(|e| e.to_string())?
    }

    async fn download(app: &AppHandle, dest: &Path) -> Result<(), String> {
        let http = reqwest::Client::builder()
            .user_agent(concat!("CubeXCraft-Installer/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|e| e.to_string())?;
        let mut response = http
            .get(ITUNES_URL)
            .send()
            .await
            .and_then(|r| r.error_for_status())
            .map_err(|e| e.without_url().to_string())?;
        let total = response.content_length().unwrap_or(0).max(1);
        let mut file = std::fs::File::create(dest).map_err(|e| e.to_string())?;
        let mut done = 0u64;
        let mut last = u8::MAX;
        set_state(app, "downloading", 0);
        while let Some(chunk) = response.chunk().await.map_err(|e| e.without_url().to_string())? {
            file.write_all(&chunk).map_err(|e| e.to_string())?;
            done += chunk.len() as u64;
            let pct = (done * 100 / total).min(100) as u8;
            if pct != last {
                last = pct;
                set_state(app, "downloading", pct);
            }
        }
        file.flush().map_err(|e| e.to_string())?;
        info!("applesvc: downloaded iTunes64Setup.exe, {done} bytes");
        Ok(())
    }

    // в iTunes64Setup.exe — CAB (iTunes64.msi, AppleMobileDeviceSupport64.msi, SetupAdmin.exe): вырезаем его и
    // достаём только нужный MSI встроенным expand.exe
    fn extract_msi(setup: &Path, dir: &Path) -> Result<PathBuf, String> {
        let mut file = std::fs::File::open(setup).map_err(|e| e.to_string())?;
        let mut head = vec![0u8; 4 << 20];
        let read = file.read(&mut head).map_err(|e| e.to_string())?;
        head.truncate(read);
        let offset = head
            .windows(4)
            .position(|w| w == b"MSCF")
            .ok_or("no CAB inside the Apple installer")?;
        let size = u32::from_le_bytes(head[offset + 8..offset + 12].try_into().unwrap()) as u64;
        let cab = dir.join("apple.cab");
        file.seek(SeekFrom::Start(offset as u64)).map_err(|e| e.to_string())?;
        let mut out = std::fs::File::create(&cab).map_err(|e| e.to_string())?;
        let copied = std::io::copy(&mut file.take(size), &mut out).map_err(|e| e.to_string())?;
        if copied != size {
            return Err("the Apple installer is truncated".into());
        }
        drop(out);
        let expand = PathBuf::from(std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".into()))
            .join(r"System32\expand.exe");
        let status = std::process::Command::new(expand)
            .arg(format!("-F:{MSI_NAME}"))
            .arg(&cab)
            .arg(dir)
            .creation_flags(CREATE_NO_WINDOW)
            .status()
            .map_err(|e| e.to_string())?;
        let msi = dir.join(MSI_NAME);
        if !status.success() || !msi.exists() {
            return Err(format!("expand.exe: {status}"));
        }
        let _ = std::fs::remove_file(&cab);
        Ok(msi)
    }

    fn powershell(script: &str) -> Result<std::process::Output, String> {
        std::process::Command::new("powershell.exe")
            .args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", script])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|e| e.to_string())
    }

    fn ps_quote(path: &Path) -> String {
        format!("'{}'", path.to_string_lossy().replace('\'', "''"))
    }

    // ставим только то, что подписано Apple
    fn check_apple_signature(msi: &Path) -> Result<(), String> {
        let out = powershell(&format!(
            "$s = Get-AuthenticodeSignature -LiteralPath {}; \
             if ($s.Status -eq 'Valid' -and $s.SignerCertificate.Subject -like '*O=Apple Inc.*') {{ 'ok' }} else {{ $s.Status }}",
            ps_quote(msi)
        ))?;
        let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if text == "ok" {
            Ok(())
        } else {
            Err(format!("Apple signature check failed: {text}"))
        }
    }

    // msiexec с правами администратора: Windows спрашивает разрешение один раз
    fn install_msi_elevated(msi: &Path) -> Result<(), String> {
        let msi_arg = format!("\"{}\"", msi.to_string_lossy()).replace('\'', "''");
        let out = powershell(&format!(
            "try {{ $p = Start-Process -FilePath msiexec.exe -ArgumentList @('/i', '{msi_arg}', '/qn', '/norestart') \
             -Verb RunAs -Wait -PassThru; exit $p.ExitCode }} catch {{ exit 1223 }}"
        ))?;
        match out.status.code() {
            Some(0) | Some(3010) => Ok(()),
            Some(1223) => Err("the administrator prompt was declined".into()),
            code => Err(format!("msiexec exit code {code:?}")),
        }
    }

    fn winget_apple_devices() -> bool {
        let output = std::process::Command::new("winget")
            .args([
                "install",
                "--id",
                APPLE_DEVICES_STORE_ID,
                "--source",
                "msstore",
                "--exact",
                "--silent",
                "--accept-package-agreements",
                "--accept-source-agreements",
                "--disable-interactivity",
            ])
            .creation_flags(CREATE_NO_WINDOW)
            .output();
        match output {
            Ok(out) => {
                info!("applesvc: winget Apple Devices: {}", out.status);
                out.status.success()
            }
            Err(e) => {
                warn!("applesvc: winget is not available: {e}");
                false
            }
        }
    }
}

pub fn start(app: AppHandle) {
    #[cfg(windows)]
    tauri::async_runtime::spawn(async move { imp::ensure(&app).await });
    #[cfg(not(windows))]
    let _ = app;
}

// «Попробовать снова» из окна
#[tauri::command]
pub async fn apple_service_install(app: AppHandle) {
    #[cfg(windows)]
    imp::ensure(&app).await;
    #[cfg(not(windows))]
    let _ = app;
}
