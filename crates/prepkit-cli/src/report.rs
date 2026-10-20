//! What the CLI prints: a line per file, or one JSON object at the end with `--json`.

use std::io::Write;
use std::process::ExitCode;

use prepkit_core::batch::{FileResult, Status, Summary};
use prepkit_core::ffmpeg::{FfmpegInfo, InstallProgress, Stage};
use serde_json::json;

pub fn file(json: bool, result: &FileResult) {
    if json {
        return;
    }
    let label = match result.status {
        Status::Written => "written",
        Status::Skipped => "skipped",
        Status::Failed => "FAILED ",
    };
    let target = result.output.as_ref().map(|out| format!(" -> {}", out.display())).unwrap_or_default();
    println!("{label}  {}{target}  ({})", result.path.display(), result.message);
}

pub fn summary(json: bool, tool: &str, summary: &Summary) -> ExitCode {
    if json {
        println!("{}", json!({ "tool": tool, "ok": summary.failed == 0 && summary.total > 0, "summary": summary }));
    } else if summary.total == 0 {
        eprintln!("No matching files found.");
    } else {
        println!(
            "{tool}: {} written, {} skipped, {} failed (of {})",
            summary.written, summary.skipped, summary.failed, summary.total
        );
    }
    if summary.total == 0 || summary.failed > 0 {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

pub fn error(json: bool, message: &str) -> ExitCode {
    if json {
        println!("{}", json!({ "ok": false, "error": message }));
    } else {
        eprintln!("{message}");
    }
    ExitCode::FAILURE
}

pub fn install_progress(json: bool, progress: &InstallProgress) {
    if json {
        return;
    }
    let mut err = std::io::stderr();
    match (progress.stage, progress.total) {
        (Stage::Downloading, Some(total)) => {
            let percent = progress.downloaded * 100 / total.max(1);
            let _ = write!(err, "\rDownloading ffmpeg… {percent}%");
        }
        (Stage::Downloading, None) => {
            let _ = write!(err, "\rDownloading ffmpeg… {} MB", progress.downloaded / 1_048_576);
        }
        (stage, _) => {
            let _ = writeln!(err, "\n{stage:?}…");
        }
    }
}

pub fn install(json: bool, result: anyhow::Result<FfmpegInfo>) -> ExitCode {
    match result {
        Ok(info) if json => {
            println!("{}", json!({ "ok": true, "ffmpeg": info }));
            ExitCode::SUCCESS
        }
        Ok(info) => {
            println!("\nInstalled ffmpeg {} at {}", info.version, info.path.display());
            println!("Open a new terminal for PATH to pick it up.");
            ExitCode::SUCCESS
        }
        Err(error) => self::error(json, &format!("{error:#}")),
    }
}
