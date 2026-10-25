// CubeXCraft (windows-claude, SCRUM-363): быстрая установка на iPhone. isideload шлёт подписанное приложение
// распакованным, файл за файлом: CubeXCraft — 208 МБ, а по Wi-Fi это ~1 МБ/с, 3,5 минуты из пяти (лог владельца
// 25.10). Подписываем так же, но шлём одним сжатым IPA (~72 МБ) — installd распакует сам. Не принял IPA — старый
// путь, папкой.
use std::{
    io::{Cursor, Write},
    path::{Path, PathBuf},
};

use idevice::{
    IdeviceService, afc::AfcClient, afc::opcode::AfcFopenMode,
    installation_proxy::InstallationProxyClient, provider::UsbmuxdProvider,
};
use isideload::{SideloadError, dev::devices::DevicesApi, util::device::IdeviceInfo};
use rootcause::prelude::*;
use tracing::{info, warn};
use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

use crate::{error::AppError, sideload::SideloaderGuard};

const UPLOAD_CHUNK: usize = 1024 * 1024;

// этап (sign, pack, upload, upload_wifi, install) и процент — для окна
pub type Progress<'a> = &'a (dyn Fn(&str, u8) + Send + Sync);

pub struct Packed {
    dir: PathBuf,
    ipa: Vec<u8>,
    bundle_id: String,
}

// подписать как isideload (устройство в команде, App ID с Increased Memory Limit, профиль) и сжать в IPA
pub async fn sign_and_pack(
    sideloader: &mut SideloaderGuard<'_>,
    provider: &UsbmuxdProvider,
    app_path: PathBuf,
    pairing: &[u8],
    progress: Progress<'_>,
) -> Result<Packed, AppError> {
    progress("sign", 0);
    let s = sideloader.get_mut();
    let device = IdeviceInfo::from_device(provider).await?;
    let team = s.get_team().await?;
    s.get_dev_session()
        .ensure_device_registered(&team, &device.name, &device.udid, None)
        .await?;
    let (dir, _) = s
        .sign_app(
            app_path,
            Some(team),
            true,
            None::<fn(f32) -> std::future::Ready<()>>,
            Some(pairing),
            Some(&device.udid),
        )
        .await?;

    progress("pack", 0);
    let bundle_id = plist::Value::from_file(dir.join("Info.plist"))
        .ok()
        .and_then(|v| {
            v.as_dictionary()?
                .get("CFBundleIdentifier")?
                .as_string()
                .map(str::to_string)
        })
        .unwrap_or_default();
    let started = std::time::Instant::now();
    let pack_dir = dir.clone();
    let ipa = tauri::async_runtime::spawn_blocking(move || pack_ipa(&pack_dir))
        .await
        .map_err(|e| AppError::Misc(format!("Packing the app failed: {e}")))?
        .map_err(|e| AppError::Misc(format!("Packing the app failed: {e}")))?;
    info!(
        "fastinstall: packed {} into {} MB in {:.1}s",
        dir.display(),
        ipa.len() >> 20,
        started.elapsed().as_secs_f32()
    );
    Ok(Packed { dir, ipa, bundle_id })
}

// передать IPA одним файлом и поставить; iPhone не принял — старый путь (папка файл за файлом)
pub async fn transfer(
    provider: &UsbmuxdProvider,
    packed: Packed,
    wifi: bool,
    progress: Progress<'_>,
) -> Result<(), AppError> {
    let upload_phase = if wifi { "upload_wifi" } else { "upload" };
    let started = std::time::Instant::now();
    let result = match install_ipa(provider, &packed, upload_phase, progress).await {
        Ok(()) => Ok(()),
        Err(e) => {
            warn!("fastinstall: IPA install failed, falling back to folder upload: {e:?}");
            progress(upload_phase, 0);
            isideload::sideload::install::install_app(provider, &packed.dir, |pct| {
                if pct < 70 {
                    progress(upload_phase, (pct * 100 / 70) as u8);
                } else {
                    progress("install", ((pct - 70) * 100 / 30) as u8);
                }
            })
            .await
            .map_err(AppError::from)
        }
    };
    info!(
        "fastinstall: {} done in {:.1}s ({})",
        packed.bundle_id,
        started.elapsed().as_secs_f32(),
        if result.is_ok() { "ok" } else { "failed" }
    );
    if let Err(e) = std::fs::remove_dir_all(&packed.dir) {
        warn!("fastinstall: failed to remove signed app: {e}");
    }
    result
}

// всё сразу — для автопродления
pub async fn install(
    sideloader: &mut SideloaderGuard<'_>,
    provider: &UsbmuxdProvider,
    app_path: PathBuf,
    pairing: &[u8],
    wifi: bool,
) -> Result<(), AppError> {
    let packed = sign_and_pack(sideloader, provider, app_path, pairing, &|_, _| {}).await?;
    transfer(provider, packed, wifi, &|_, _| {}).await
}

async fn install_ipa(
    provider: &UsbmuxdProvider,
    packed: &Packed,
    upload_phase: &str,
    progress: Progress<'_>,
) -> Result<(), Report> {
    let name = packed
        .dir
        .file_name()
        .map(|n| n.to_string_lossy().trim_end_matches(".app").to_string())
        .unwrap_or_else(|| "app".into());
    let staging = format!("PublicStaging/{name}.ipa");

    let mut afc = AfcClient::connect(provider)
        .await
        .map_err(SideloadError::IdeviceError)?;
    let _ = afc.mk_dir("PublicStaging").await;
    {
        let mut file = afc
            .open(staging.clone(), AfcFopenMode::WrOnly)
            .await
            .map_err(SideloadError::IdeviceError)?;
        let total = packed.ipa.len().max(1);
        let mut sent = 0;
        let mut last = u8::MAX;
        for chunk in packed.ipa.chunks(UPLOAD_CHUNK) {
            file.write_entire(chunk)
                .await
                .map_err(SideloadError::IdeviceError)?;
            sent += chunk.len();
            let pct = (sent * 100 / total) as u8;
            if pct != last {
                last = pct;
                progress(upload_phase, pct);
            }
        }
        file.close().await.map_err(SideloadError::IdeviceError)?;
    }

    progress("install", 0);
    let mut instproxy = InstallationProxyClient::connect(provider)
        .await
        .map_err(SideloadError::IdeviceError)?;
    let mut options = plist::Dictionary::new();
    if !packed.bundle_id.is_empty() {
        options.insert("CFBundleIdentifier".into(), packed.bundle_id.clone().into());
    }
    let result = instproxy
        .install_with_callback(
            staging.clone(),
            Some(plist::Value::Dictionary(options)),
            async |(pct, _)| progress("install", pct as u8),
            (),
        )
        .await
        .map_err(SideloadError::IdeviceError);
    let _ = afc.remove(staging).await;
    result?;
    Ok(())
}

fn pack_ipa(app_dir: &Path) -> Result<Vec<u8>, Report> {
    let name = app_dir
        .file_name()
        .ok_or_else(|| report!("Signed app has no name"))?
        .to_string_lossy()
        .to_string();
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    let dir_options = SimpleFileOptions::default().unix_permissions(0o755);
    zip.add_directory("Payload/", dir_options)?;
    add_dir(&mut zip, app_dir, &format!("Payload/{name}"))?;
    Ok(zip.finish()?.into_inner())
}

fn add_dir(zip: &mut ZipWriter<Cursor<Vec<u8>>>, dir: &Path, prefix: &str) -> Result<(), Report> {
    zip.add_directory(
        format!("{prefix}/"),
        SimpleFileOptions::default().unix_permissions(0o755),
    )?;
    let mut entries = std::fs::read_dir(dir)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let path = entry.path();
        let child = format!("{prefix}/{}", entry.file_name().to_string_lossy());
        if std::fs::metadata(&path)?.is_dir() {
            add_dir(zip, &path, &child)?;
        } else {
            let bytes = std::fs::read(&path)?;
            // исполняемые Mach-O — с правом запуска, как в IPA от Xcode
            let mode = if is_macho(&bytes) { 0o755 } else { 0o644 };
            let options = SimpleFileOptions::default()
                .compression_method(CompressionMethod::Deflated)
                .unix_permissions(mode)
                .large_file(bytes.len() >= u32::MAX as usize);
            zip.start_file(child, options)?;
            zip.write_all(&bytes)?;
        }
    }
    Ok(())
}

fn is_macho(bytes: &[u8]) -> bool {
    matches!(
        bytes.get(..4),
        Some([0xcf, 0xfa, 0xed, 0xfe] | [0xce, 0xfa, 0xed, 0xfe] | [0xca, 0xfe, 0xba, 0xbe])
    )
}

// подписали, но ставить не будем — убрать за собой
pub fn discard(packed: Packed) {
    let _ = std::fs::remove_dir_all(&packed.dir);
}
