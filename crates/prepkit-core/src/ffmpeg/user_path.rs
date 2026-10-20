//! Adds a folder to the user's PATH (`HKCU\Environment\Path`) and tells Windows, so new terminals see it.
//! Only the per-user PATH is touched; no admin rights are needed.

use std::path::Path;

/// `current` with `entry` appended, or `None` when it's already there (case and trailing slashes ignored).
pub fn with_entry(current: &str, entry: &str) -> Option<String> {
    let normalize = |value: &str| value.trim().trim_end_matches(['\\', '/']).to_ascii_lowercase();
    let wanted = normalize(entry);
    if current.split(';').any(|part| normalize(part) == wanted) {
        return None;
    }
    let base = current.trim_end_matches(';');
    Some(if base.is_empty() { entry.to_string() } else { format!("{base};{entry}") })
}

/// Adds `dir` to the user's PATH. Returns whether it changed anything.
#[cfg(windows)]
pub fn add(dir: &Path) -> anyhow::Result<bool> {
    use anyhow::Context;
    use winreg::enums::{RegType, HKEY_CURRENT_USER, KEY_READ, KEY_WRITE};
    use winreg::{RegKey, RegValue};

    let environment = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)
        .context("Couldn't open the user environment")?;
    let (current, kind) = match environment.get_raw_value("Path") {
        Ok(value) => (decode(&value.bytes), value.vtype),
        Err(_) => (String::new(), RegType::REG_EXPAND_SZ),
    };
    let Some(updated) = with_entry(&current, &dir.to_string_lossy()) else { return Ok(false) };

    let kind = if matches!(kind, RegType::REG_SZ) { RegType::REG_SZ } else { RegType::REG_EXPAND_SZ };
    environment
        .set_raw_value("Path", &RegValue { bytes: encode(&updated), vtype: kind })
        .context("Couldn't update the user PATH")?;
    broadcast_change();
    Ok(true)
}

#[cfg(not(windows))]
pub fn add(_dir: &Path) -> anyhow::Result<bool> {
    anyhow::bail!("Adding to PATH is Windows-only")
}

#[cfg(windows)]
fn decode(bytes: &[u8]) -> String {
    let wide: Vec<u16> = bytes.chunks_exact(2).map(|pair| u16::from_le_bytes([pair[0], pair[1]])).collect();
    String::from_utf16_lossy(&wide).trim_end_matches('\0').to_string()
}

#[cfg(windows)]
fn encode(value: &str) -> Vec<u8> {
    value.encode_utf16().chain(std::iter::once(0)).flat_map(u16::to_le_bytes).collect()
}

/// Tells Explorer and other programs that environment variables changed.
#[cfg(windows)]
fn broadcast_change() {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG, WM_SETTINGCHANGE,
    };
    let area: Vec<u16> = "Environment\0".encode_utf16().collect();
    let mut result = 0usize;
    unsafe {
        SendMessageTimeoutW(
            HWND_BROADCAST,
            WM_SETTINGCHANGE,
            0,
            area.as_ptr() as isize,
            SMTO_ABORTIFHUNG,
            5000,
            &mut result,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn appends_once() {
        assert_eq!(with_entry(r"C:\a;C:\b", r"C:\ffmpeg\bin").unwrap(), r"C:\a;C:\b;C:\ffmpeg\bin");
        assert_eq!(with_entry(r"C:\a;C:\FFMPEG\bin\", r"c:\ffmpeg\bin"), None);
    }

    #[test]
    fn handles_empty_and_trailing_semicolons() {
        assert_eq!(with_entry("", r"C:\x").unwrap(), r"C:\x");
        assert_eq!(with_entry(r"C:\a;", r"C:\x").unwrap(), r"C:\a;C:\x");
    }
}
