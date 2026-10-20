//! Downloads ffmpeg (BtbN's Windows build), checks its SHA256, unpacks `bin/` to
//! `%LOCALAPPDATA%\Programs\ffmpeg\bin` and adds that folder to the user's PATH.

use std::env;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::Serialize;
use sha2::{Digest, Sha256};

use super::locate::{version_of, FfmpegInfo};
use super::user_path;

const RELEASE: &str = "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest";
const ZIP_NAME: &str = "ffmpeg-master-latest-win64-gpl.zip";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Stage {
    Downloading,
    Verifying,
    Extracting,
    AddingToPath,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallProgress {
    pub stage: Stage,
    pub downloaded: u64,
    pub total: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct InstallOptions {
    /// Where ffmpeg goes; its `bin/` folder is what lands on PATH.
    pub dir: PathBuf,
    /// Add `<dir>/bin` to the user's PATH (Windows registry) and to this process.
    pub add_to_path: bool,
}

impl Default for InstallOptions {
    fn default() -> Self {
        Self { dir: install_dir(), add_to_path: true }
    }
}

/// `%LOCALAPPDATA%\Programs\ffmpeg`: per user, no admin rights needed.
pub fn install_dir() -> PathBuf {
    let base = env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".local")))
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("Programs").join("ffmpeg")
}

/// Installs ffmpeg and returns the working copy.
pub fn install(options: &InstallOptions, on_progress: impl Fn(InstallProgress)) -> Result<FfmpegInfo> {
    if !cfg!(windows) {
        bail!("Automatic install is Windows-only. Install ffmpeg with your package manager.");
    }
    let report = |stage, downloaded, total| on_progress(InstallProgress { stage, downloaded, total });
    fs::create_dir_all(&options.dir).with_context(|| format!("Couldn't create {}", options.dir.display()))?;

    let expected = expected_sha256()?;
    let zip_path = options.dir.join("ffmpeg-download.zip");
    let actual = download(&format!("{RELEASE}/{ZIP_NAME}"), &zip_path, |downloaded, total| {
        report(Stage::Downloading, downloaded, total)
    })?;

    report(Stage::Verifying, 0, None);
    if !actual.eq_ignore_ascii_case(&expected) {
        let _ = fs::remove_file(&zip_path);
        bail!("The download doesn't match its published SHA256, so it wasn't installed.");
    }

    report(Stage::Extracting, 0, None);
    let bin = options.dir.join("bin");
    extract_bin(&zip_path, &options.dir, &bin)?;
    let _ = fs::remove_file(&zip_path);

    let exe = bin.join("ffmpeg.exe");
    let version = version_of(&exe).context("ffmpeg was unpacked but doesn't run")?;

    if options.add_to_path {
        report(Stage::AddingToPath, 0, None);
        user_path::add(&bin)?;
        add_to_process_path(&bin);
    }
    Ok(FfmpegInfo { path: exe, version })
}

/// The SHA256 that BtbN publishes for the zip.
fn expected_sha256() -> Result<String> {
    let sums = ureq::get(&format!("{RELEASE}/checksums.sha256"))
        .call()
        .context("Couldn't reach GitHub for ffmpeg's checksums")?
        .into_string()?;
    sums.lines()
        .find(|line| line.trim_end().ends_with(ZIP_NAME))
        .and_then(|line| line.split_whitespace().next())
        .map(str::to_string)
        .context("The checksum list doesn't mention the ffmpeg zip")
}

/// Streams `url` into `path`, returning the file's SHA256 (hex).
fn download(url: &str, path: &Path, on_progress: impl Fn(u64, Option<u64>)) -> Result<String> {
    let response = ureq::get(url).call().context("Couldn't download ffmpeg")?;
    let total = response.header("Content-Length").and_then(|value| value.parse().ok());
    let mut reader = response.into_reader();
    let mut file = File::create(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; 256 * 1024];
    let mut downloaded = 0u64;
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        file.write_all(&buffer[..read])?;
        hasher.update(&buffer[..read]);
        downloaded += read as u64;
        on_progress(downloaded, total);
    }
    file.flush()?;
    Ok(hasher.finalize().iter().map(|byte| format!("{byte:02x}")).collect())
}

/// Copies every file under the zip's `*/bin/` into `bin`, and its LICENSE next to it.
fn extract_bin(zip_path: &Path, dir: &Path, bin: &Path) -> Result<()> {
    let mut archive = zip::ZipArchive::new(File::open(zip_path)?).context("The download isn't a valid zip")?;
    fs::create_dir_all(bin)?;
    let mut found = false;
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;
        let Some(name) = entry.enclosed_name() else { continue };
        let parts: Vec<_> = name.components().map(|c| c.as_os_str().to_string_lossy().to_string()).collect();
        let target = match parts.as_slice() {
            [_, folder, file] if folder == "bin" && entry.is_file() => bin.join(file),
            [_, file] if file.eq_ignore_ascii_case("LICENSE.txt") => dir.join("LICENSE.txt"),
            _ => continue,
        };
        let mut out = File::create(&target).with_context(|| format!("Couldn't write {}", target.display()))?;
        io::copy(&mut entry, &mut out)?;
        found |= target.file_name().is_some_and(|f| f == "ffmpeg.exe");
    }
    if !found {
        bail!("The zip had no bin/ffmpeg.exe");
    }
    Ok(())
}

fn add_to_process_path(bin: &Path) {
    let current = env::var_os("PATH").unwrap_or_default();
    let mut paths: Vec<PathBuf> = env::split_paths(&current).collect();
    if !paths.iter().any(|path| path == bin) {
        paths.insert(0, bin.to_path_buf());
        if let Ok(joined) = env::join_paths(paths) {
            env::set_var("PATH", joined);
        }
    }
}
