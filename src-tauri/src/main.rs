// No console window behind the app in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod files;
mod jobs;
mod settings;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(jobs::Jobs::default())
        .invoke_handler(tauri::generate_handler![
            commands::run_pixel_fix,
            commands::run_audio_convert,
            commands::cancel_job,
            commands::ffmpeg_status,
            commands::install_ffmpeg,
            files::list_images,
            files::list_audio,
            settings::load_settings,
            settings::save_settings,
        ])
        .run(tauri::generate_context!())
        .expect("PrepKit failed to start");
}
