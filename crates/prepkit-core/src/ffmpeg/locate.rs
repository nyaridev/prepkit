//! Looks for a working ffmpeg: `PREPKIT_FFMPEG`, then PATH, then PrepKit's own install folder (which a terminal
//! opened before the install doesn't have on its PATH yet).

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Serialize;

use super::install::install_dir;

const EXE: &str = if cfg!(windows) { "ffmpeg.exe" } else { "ffmpeg" };

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FfmpegInfo {
    pub path: PathBuf,
    pub version: String,
}

/// The first ffmpeg that actually runs.
pub fn locate() -> Option<FfmpegInfo> {
    candidates().into_iter().find_map(|path| {
        let version = version_of(&path)?;
        Some(FfmpegInfo { path, version })
    })
}

fn candidates() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Some(custom) = env::var_os("PREPKIT_FFMPEG") {
        paths.push(PathBuf::from(custom));
    }
    if let Some(path) = env::var_os("PATH") {
        paths.extend(env::split_paths(&path).map(|dir| dir.join(EXE)));
    }
    paths.push(install_dir().join("bin").join(EXE));
    paths.into_iter().filter(|path| path.is_file()).collect()
}

/// The version from `ffmpeg -version` ("ffmpeg version 7.1 Copyright…" → "7.1"), or `None` if it doesn't run.
pub fn version_of(path: &Path) -> Option<String> {
    let output = command(path).arg("-version").output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let first = text.lines().next()?;
    Some(short_version(first.split_whitespace().nth(2).unwrap_or("unknown")))
}

/// Release builds stay as they are ("7.1"); nightly builds ("N-118858-g740d400965-20250318") become
/// "nightly 2025-03-18".
fn short_version(raw: &str) -> String {
    let Some((_, rest)) = raw.split_once("-g") else { return raw.to_string() };
    match rest.rsplit_once('-') {
        Some((_, date)) if date.len() == 8 && date.bytes().all(|b| b.is_ascii_digit()) => {
            format!("nightly {}-{}-{}", &date[..4], &date[4..6], &date[6..])
        }
        _ => raw.split("-g").next().unwrap_or(raw).to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::short_version;

    #[test]
    fn shortens_nightly_versions() {
        assert_eq!(short_version("N-118858-g740d400965-20250318"), "nightly 2025-03-18");
        assert_eq!(short_version("7.1"), "7.1");
        assert_eq!(short_version("N-1-gabc"), "N-1");
    }
}

/// A `Command` for ffmpeg that never opens a console window (PrepKit's window is a GUI app on Windows).
pub fn command(path: &Path) -> Command {
    #[allow(unused_mut)]
    let mut command = Command::new(path);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    command
}
