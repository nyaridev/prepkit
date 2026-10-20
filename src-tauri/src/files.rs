//! Turns the files and folders the user added into the list of files a tool will process, so the window can show
//! them (with image previews) before anything runs.

use std::path::PathBuf;

use prepkit_core::audio;
use prepkit_core::pixel_fix::{self, PixelFixOptions};
use tauri::{AppHandle, Manager};

/// The images a Pixel Fix run would touch. Each one is also opened to the asset protocol, so the window can show
/// it as a thumbnail; nothing else on disk is readable that way.
#[tauri::command]
pub async fn list_images(app: AppHandle, paths: Vec<PathBuf>, options: PixelFixOptions) -> Vec<PathBuf> {
    let files = tauri::async_runtime::spawn_blocking(move || pixel_fix::discover(&paths, &options))
        .await
        .unwrap_or_default();
    let scope = app.asset_protocol_scope();
    for file in &files {
        let _ = scope.allow_file(file);
    }
    files
}

/// The audio files an Audio Convert run would process.
#[tauri::command]
pub async fn list_audio(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    tauri::async_runtime::spawn_blocking(move || audio::discover(&paths)).await.unwrap_or_default()
}
