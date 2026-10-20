//! The window's settings, as a JSON object the frontend owns (it fills in defaults for anything missing).
//! Stored in `%APPDATA%\com.nyaridev.prepkit\settings.json`. Folders and files are never saved.

use std::fs;
use std::path::PathBuf;

use serde_json::Value;
use tauri::{AppHandle, Manager};

fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_config_dir().map_err(|error| error.to_string())?;
    Ok(dir.join("settings.json"))
}

/// The saved settings, or `null` when there are none yet (or the file is broken).
#[tauri::command]
pub fn load_settings(app: AppHandle) -> Value {
    settings_path(&app)
        .ok()
        .and_then(|path| fs::read_to_string(path).ok())
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or(Value::Null)
}

#[tauri::command]
pub fn save_settings(app: AppHandle, settings: Value) -> Result<(), String> {
    if !settings.is_object() {
        return Err("Settings must be an object".into());
    }
    let path = settings_path(&app)?;
    let dir = path.parent().ok_or("No settings folder")?;
    fs::create_dir_all(dir).map_err(|error| error.to_string())?;
    let text = serde_json::to_string_pretty(&settings).map_err(|error| error.to_string())?;
    let temp = path.with_extension("json.tmp");
    fs::write(&temp, text).map_err(|error| error.to_string())?;
    fs::rename(&temp, &path).map_err(|error| error.to_string())
}
