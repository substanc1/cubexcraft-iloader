// CubeXCraft (windows-claude, SCRUM-363): «одна кнопка» — поставить лаунчер CubeXCraft и подготовить JIT.
// Установка — как Import IPA (с правом Increased Memory Limit), затем файл сопряжения сам кладётся в StikDebug.
// Чего на телефоне нет (StikDebug, LocalDevVPN — оба из App Store), интерфейс показывает QR-кодом на App Store.
use std::path::PathBuf;

use idevice::{
    IdeviceService, afc::opcode::AfcFopenMode, house_arrest::HouseArrestClient,
    installation_proxy::InstallationProxyClient, provider::IdeviceProvider,
};
use qrcode::{QrCode, render::svg};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Manager, State, Window};

use crate::{
    device::{DeviceInfoMutex, get_provider, is_wifi_only},
    error::AppError,
    fastinstall,
    operation::Operation,
    pairing::place_file,
    sideload::{SideloaderGuard, SideloaderMutex},
};

// куда StikDebug ищет файл сопряжения (как в списке приложений pairing.rs)
const STIKDEBUG_PAIRING_PATH: &str = "pairingFile.plist";
// свежая сборка лаунчера: версия, ссылка на IPA и её SHA-256 (рядом на сайте — исходники, GPL-3.0)
pub const IOS_MANIFEST_URL: &str = "https://www.cubexcraft.site/cxl/launcher/ios.json";

#[derive(Deserialize)]
struct IosManifest {
    version: String,
    url: String,
    sha256: String,
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect()
}

// IPA с сайта в кэш приложения; уже скачанная с той же суммой не качается заново. Возвращает путь и версию
pub async fn fetch_ipa(handle: &AppHandle) -> Result<(PathBuf, String), AppError> {
    let text = reqwest::get(IOS_MANIFEST_URL)
        .await
        .and_then(|r| r.error_for_status())
        .map_err(|e| AppError::Download(e.to_string()))?
        .text()
        .await
        .map_err(|e| AppError::Download(e.to_string()))?;
    let manifest: IosManifest =
        serde_json::from_str(&text).map_err(|e| AppError::Download(e.to_string()))?;
    let want = manifest.sha256.to_lowercase();

    let dir = handle
        .path()
        .app_cache_dir()
        .map_err(|e| AppError::Filesystem("Failed to get cache dir".into(), e.to_string()))?
        .join("cubexcraft");
    tokio::fs::create_dir_all(&dir)
        .await
        .map_err(|e| AppError::Filesystem("Failed to create cache dir".into(), e.to_string()))?;
    let dest = dir.join(format!("CubeXCraft-ios-{}.ipa", manifest.version));
    if let Ok(bytes) = tokio::fs::read(&dest).await {
        if sha256_hex(&bytes) == want {
            return Ok((dest, manifest.version));
        }
    }

    let bytes = reqwest::get(&manifest.url)
        .await
        .and_then(|r| r.error_for_status())
        .map_err(|e| AppError::Download(e.to_string()))?
        .bytes()
        .await
        .map_err(|e| AppError::Download(e.to_string()))?;
    if sha256_hex(&bytes) != want {
        return Err(AppError::Download(
            "CubeXCraft IPA checksum mismatch, try again".into(),
        ));
    }
    tokio::fs::write(&dest, &bytes)
        .await
        .map_err(|e| AppError::Filesystem("Failed to save CubeXCraft IPA".into(), e.to_string()))?;
    Ok((dest, manifest.version))
}

// StikDebug убран из App Store (25.10: нет ни в RU, ни в US) — берём последний выпуск с GitHub (AGPL-3.0)
// и ставим сами, как CubeXCraft. Сумма — из поля digest выпуска, если GitHub его отдаёт
const STIKDEBUG_RELEASE_URL: &str = "https://api.github.com/repos/StikDebug/StikDebug/releases/latest";

pub async fn fetch_stikdebug(handle: &AppHandle) -> Result<(PathBuf, String), AppError> {
    let client = reqwest::Client::builder()
        .user_agent("CubeXCraft-Installer")
        .build()
        .map_err(|e| AppError::Download(e.to_string()))?;
    let text = client
        .get(STIKDEBUG_RELEASE_URL)
        .send()
        .await
        .and_then(|r| r.error_for_status())
        .map_err(|e| AppError::Download(e.to_string()))?
        .text()
        .await
        .map_err(|e| AppError::Download(e.to_string()))?;
    let release: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| AppError::Download(e.to_string()))?;
    let version = release["tag_name"].as_str().unwrap_or("latest").to_string();
    let asset = release["assets"]
        .as_array()
        .and_then(|assets| {
            assets
                .iter()
                .find(|a| a["name"].as_str().is_some_and(|n| n.ends_with(".ipa")))
        })
        .ok_or_else(|| AppError::Download("StikDebug IPA not found in the latest release".into()))?;
    let url = asset["browser_download_url"]
        .as_str()
        .ok_or_else(|| AppError::Download("StikDebug IPA has no download URL".into()))?
        .to_string();
    let want = asset["digest"]
        .as_str()
        .and_then(|d| d.strip_prefix("sha256:"))
        .map(|d| d.to_lowercase());

    let dir = handle
        .path()
        .app_cache_dir()
        .map_err(|e| AppError::Filesystem("Failed to get cache dir".into(), e.to_string()))?
        .join("cubexcraft");
    tokio::fs::create_dir_all(&dir)
        .await
        .map_err(|e| AppError::Filesystem("Failed to create cache dir".into(), e.to_string()))?;
    let dest = dir.join(format!("StikDebug-{}.ipa", version));
    if let Ok(bytes) = tokio::fs::read(&dest).await {
        if !bytes.is_empty() && want.as_ref().is_none_or(|w| sha256_hex(&bytes) == *w) {
            return Ok((dest, version));
        }
    }

    let bytes = client
        .get(&url)
        .send()
        .await
        .and_then(|r| r.error_for_status())
        .map_err(|e| AppError::Download(e.to_string()))?
        .bytes()
        .await
        .map_err(|e| AppError::Download(e.to_string()))?;
    if want.as_ref().is_some_and(|w| sha256_hex(&bytes) != *w) {
        return Err(AppError::Download("StikDebug IPA checksum mismatch, try again".into()));
    }
    tokio::fs::write(&dest, &bytes)
        .await
        .map_err(|e| AppError::Filesystem("Failed to save StikDebug IPA".into(), e.to_string()))?;
    Ok((dest, version))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CubeXCraftStatus {
    pub cubexcraft: bool,
    pub stikdebug: Option<String>,
    pub localdevvpn: bool,
}

pub async fn device_status(provider: &dyn IdeviceProvider) -> Result<CubeXCraftStatus, AppError> {
    let mut installation_proxy = InstallationProxyClient::connect(provider).await.map_err(|e| {
        AppError::DeviceComsWithMessage(
            "Failed to connect to installation proxy".into(),
            e.to_string(),
        )
    })?;
    let apps = installation_proxy
        .get_apps(Some("User"), None)
        .await
        .map_err(|e| {
            AppError::DeviceComsWithMessage("Failed to get installed apps".into(), e.to_string())
        })?;

    let mut status = CubeXCraftStatus {
        cubexcraft: false,
        stikdebug: None,
        localdevvpn: false,
    };
    for (bundle_id, app) in apps {
        let name = app
            .as_dictionary()
            .and_then(|x| x.get("CFBundleDisplayName").and_then(|x| x.as_string()))
            .unwrap_or("");
        let id = bundle_id.to_lowercase();
        if id.starts_with("site.cubexcraft.ios") {
            status.cubexcraft = true;
        } else if name == "StikDebug" || id.contains("stikdebug") {
            status.stikdebug = Some(bundle_id);
        } else if name == "LocalDevVPN" || id.contains("localdevvpn") {
            status.localdevvpn = true;
        }
    }
    Ok(status)
}

#[tauri::command]
pub async fn cubexcraft_status(
    device_state: State<'_, DeviceInfoMutex>,
) -> Result<CubeXCraftStatus, AppError> {
    let device = {
        let device_guard = device_state.lock().unwrap();
        match &*device_guard {
            Some(d) => d.clone(),
            None => return Err(AppError::NoDeviceSelected),
        }
    };
    let provider = get_provider(&device.info).await?;
    device_status(&provider).await
}

// QR-код ссылки на App Store — SVG строкой
#[tauri::command]
pub fn cubexcraft_qr(text: String) -> Result<String, AppError> {
    let code = QrCode::new(text.as_bytes()).map_err(|e| AppError::Misc(e.to_string()))?;
    Ok(code
        .render::<svg::Color>()
        .min_dimensions(148, 148)
        .quiet_zone(true)
        .dark_color(svg::Color("#0b0509"))
        .light_color(svg::Color("#ffffff"))
        .build())
}

// app_path — свой IPA из файла; без него — свежая сборка с сайта
#[tauri::command]
pub async fn install_cubexcraft_operation(
    handle: AppHandle,
    window: Window,
    device_state: State<'_, DeviceInfoMutex>,
    sideloader_state: State<'_, SideloaderMutex>,
    app_path: Option<String>,
) -> Result<(), AppError> {
    let op = Operation::new("install_cubexcraft".to_string(), &window);
    // пока ставим — самообновление ждёт
    let _busy = crate::selfupdate::BusyGuard::new();
    let device = {
        let device_guard = device_state.lock().unwrap();
        match &*device_guard {
            Some(d) => d.clone(),
            None => return op.fail("download", AppError::NoDeviceSelected),
        }
    };

    op.start("download")?;
    let provider = op.fail_if_err("download", get_provider(&device.info).await)?;
    let wifi = is_wifi_only(&device.info.udid).await;
    // StikDebug: из App Store убран — ставим сами; наш же продлеваем при каждой установке, чтобы сроки шли вместе
    let status = op.fail_if_err("download", device_status(&provider).await)?;
    let stikdebug_ours =
        status.stikdebug.is_none() || crate::autorenew::stikdebug_is_ours(&handle, &device.info.udid);

    // CubeXCraft с сайта и StikDebug с GitHub качаем разом
    let (cubexcraft, stik) = futures::join!(
        async {
            match app_path {
                Some(path) => Ok((PathBuf::from(&path), String::new(), Some(path))),
                None => fetch_ipa(&handle).await.map(|(path, version)| (path, version, None)),
            }
        },
        async {
            if stikdebug_ours {
                fetch_stikdebug(&handle).await.map(|(path, _)| Some(path))
            } else {
                Ok(None)
            }
        },
    );
    let (app_path, version, file) = op.fail_if_err("download", cubexcraft)?;
    op.move_on("download", "install")?;

    let mut sideloader = op.fail_if_err("install", SideloaderGuard::take(&sideloader_state))?;
    let email = sideloader.get_mut().get_email().to_string();
    let install_progress = |phase: &str, pct: u8| op.progress("install", phase, pct);
    let stik_progress = |phase: &str, pct: u8| op.progress("stikdebug", phase, pct);
    let packed = op.fail_if_err(
        "install",
        fastinstall::sign_and_pack(&mut sideloader, &provider, app_path, &device.pairing, &install_progress)
            .await,
    )?;

    // пока CubeXCraft едет на iPhone, подписываем StikDebug
    let (installed, stik_packed) = futures::join!(
        fastinstall::transfer(&provider, packed, wifi, &install_progress),
        async {
            match stik {
                Ok(Some(path)) => fastinstall::sign_and_pack(
                    &mut sideloader,
                    &provider,
                    path,
                    &device.pairing,
                    &|_, _| {},
                )
                .await
                .map(Some),
                Ok(None) => Ok(None),
                Err(e) => Err(e),
            }
        },
    );
    if let Err(e) = installed {
        if let Ok(Some(packed)) = stik_packed {
            fastinstall::discard(packed);
        }
        return op.fail("install", e);
    }
    op.move_on("install", "stikdebug")?;
    if let Some(packed) = op.fail_if_err("stikdebug", stik_packed)? {
        op.fail_if_err(
            "stikdebug",
            fastinstall::transfer(&provider, packed, wifi, &stik_progress).await,
        )?;
    }
    drop(sideloader);

    // этот iPhone дальше продлевает автопродление (SCRUM-368)
    crate::autorenew::record_install(&handle, &device.info, &email, &version, file, stikdebug_ours);
    op.move_on("stikdebug", "apps")?;

    // LocalDevVPN ставится только из App Store — ждём, пока игрок поставит его по QR-коду
    // (окно показывает код, пока шаг идёт), и продолжаем сами: «одна кнопка» (владелец 25.10)
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30 * 60);
    let stikdebug = loop {
        if let Ok(status) = device_status(&provider).await {
            if let (Some(stikdebug), true) = (status.stikdebug, status.localdevvpn) {
                break stikdebug;
            }
        }
        if std::time::Instant::now() > deadline {
            return op.fail(
                "apps",
                AppError::Misc(
                    "StikDebug or LocalDevVPN is still not installed: install them from the App Store, then press the button again".into(),
                ),
            );
        }
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    };
    op.move_on("apps", "pairing")?;
    // JIT без вопроса в StikDebug; не вышло — не беда, StikDebug просто спросит
    if let Err(e) = stikdebug_skip_confirm(&provider, &stikdebug).await {
        tracing::warn!("StikDebug: could not turn off «Confirm JIT Links»: {e:?}");
    }
    op.fail_if_err(
        "pairing",
        place_file(
            device.pairing,
            &provider,
            stikdebug,
            STIKDEBUG_PAIRING_PATH.to_string(),
        )
        .await,
    )?;
    op.complete("pairing")?;
    Ok(())
}

// Владелец 25.10 (вариант 1): CubeXCraft открывает StikDebug ссылкой stikjit://enable-jit, и JIT включается сам,
// без окна «Confirm JIT Links». Это ключ confirmExternalJITRequests в его UserDefaults — пишем файл настроек в
// контейнер (house_arrest VendContainer: StikDebug подписан нашим разработческим сертификатом). Минус: без вопроса
// StikDebug выполнит и чужую ссылку stikjit://. Если StikDebug уже запускался, iOS может держать прежнее значение
// в памяти до перезагрузки.
const STIKDEBUG_CONFIRM_KEY: &str = "confirmExternalJITRequests";

pub async fn stikdebug_skip_confirm(provider: &dyn IdeviceProvider, bundle_id: &str) -> Result<(), AppError> {
    let house_arrest = HouseArrestClient::connect(provider).await.map_err(|e| {
        AppError::HouseArrest("Failed to connect to house arrest".into(), e.to_string())
    })?;
    let mut afc = house_arrest
        .vend_container(bundle_id)
        .await
        .map_err(|e| AppError::HouseArrest("Failed to vend container".into(), e.to_string()))?;
    let path = format!("/Library/Preferences/{bundle_id}.plist");
    let mut prefs = match afc.open(path.clone(), AfcFopenMode::RdOnly).await {
        Ok(mut file) => {
            let bytes = file.read_entire().await.unwrap_or_default();
            let _ = file.close().await;
            plist::Value::from_reader(std::io::Cursor::new(bytes))
                .ok()
                .and_then(plist::Value::into_dictionary)
                .unwrap_or_default()
        }
        Err(_) => plist::Dictionary::new(),
    };
    if prefs.get(STIKDEBUG_CONFIRM_KEY).and_then(plist::Value::as_boolean) == Some(false) {
        return Ok(());
    }
    prefs.insert(STIKDEBUG_CONFIRM_KEY.into(), false.into());
    let mut bytes = Vec::new();
    plist::Value::Dictionary(prefs)
        .to_writer_binary(&mut bytes)
        .map_err(|e| AppError::Misc(format!("StikDebug settings: {e}")))?;
    let _ = afc.mk_dir("/Library/Preferences").await;
    let mut file = afc
        .open(path, AfcFopenMode::WrOnly)
        .await
        .map_err(|e| AppError::HouseArrest("Failed to open StikDebug settings".into(), e.to_string()))?;
    file.write_entire(&bytes)
        .await
        .map_err(|e| AppError::HouseArrest("Failed to write StikDebug settings".into(), e.to_string()))?;
    file.close()
        .await
        .map_err(|e| AppError::HouseArrest("Failed to write StikDebug settings".into(), e.to_string()))?;
    tracing::info!("StikDebug: «Confirm JIT Links» turned off");
    Ok(())
}
