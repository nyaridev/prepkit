//! What the window calls: run a tool (with progress events), cancel it, and check or install ffmpeg.

use std::path::PathBuf;

use prepkit_core::audio::{self, AudioOptions};
use prepkit_core::batch::{FileResult, Summary};
use prepkit_core::ffmpeg::{self, FfmpegInfo, InstallOptions, InstallProgress};
use prepkit_core::pixel_fix::{self, PixelFixOptions};
use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

use crate::jobs::Jobs;

/// Sent as `job-progress` after every file.
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Progress<'a> {
    job: &'a str,
    result: &'a FileResult,
    done: usize,
    total: usize,
}

fn emit_progress(app: &AppHandle, job: &str, result: &FileResult, done: usize, total: usize) {
    let _ = app.emit("job-progress", Progress { job, result, done, total });
}

#[tauri::command]
pub async fn run_pixel_fix(
    app: AppHandle,
    jobs: State<'_, Jobs>,
    paths: Vec<PathBuf>,
    options: PixelFixOptions,
) -> Result<Summary, String> {
    let cancel = jobs.start("pixel-fix");
    tauri::async_runtime::spawn_blocking(move || {
        pixel_fix::run(&paths, &options, &cancel, |result, done, total| {
            emit_progress(&app, "pixel-fix", result, done, total)
        })
    })
    .await
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn run_audio_convert(
    app: AppHandle,
    jobs: State<'_, Jobs>,
    paths: Vec<PathBuf>,
    options: AudioOptions,
) -> Result<Summary, String> {
    let ffmpeg = ffmpeg::locate().ok_or_else(|| "ffmpeg wasn't found. Use Download ffmpeg first.".to_string())?;
    let cancel = jobs.start("audio-convert");
    tauri::async_runtime::spawn_blocking(move || {
        audio::run(&paths, &options, &ffmpeg.path, &cancel, |result, done, total| {
            emit_progress(&app, "audio-convert", result, done, total)
        })
    })
    .await
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn cancel_job(jobs: State<'_, Jobs>, job: String) {
    jobs.cancel(&job);
}

#[tauri::command]
pub async fn ffmpeg_status() -> Option<FfmpegInfo> {
    tauri::async_runtime::spawn_blocking(ffmpeg::locate).await.ok().flatten()
}

/// Downloads ffmpeg, adds it to the user's PATH, and reports `ffmpeg-install-progress` along the way.
#[tauri::command]
pub async fn install_ffmpeg(app: AppHandle) -> Result<FfmpegInfo, String> {
    tauri::async_runtime::spawn_blocking(move || {
        ffmpeg::install(&InstallOptions::default(), |progress: InstallProgress| {
            let _ = app.emit("ffmpeg-install-progress", progress);
        })
        .map_err(|error| format!("{error:#}"))
    })
    .await
    .map_err(|error| error.to_string())?
}
