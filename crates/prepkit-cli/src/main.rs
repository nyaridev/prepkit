//! `prepkit-cli`: the same tools as the PrepKit window, for scripts and AI agents.
//! Defaults match the app's defaults; the app's saved settings are never read, so runs are reproducible.

mod report;

use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::atomic::AtomicBool;

use clap::{Parser, Subcommand, ValueEnum};
use prepkit_core::audio::{self, AudioOptions, Mode};
use prepkit_core::ffmpeg::{self, InstallOptions};
use prepkit_core::pixel_fix::{self, Output, PixelFixOptions};

#[derive(Parser)]
#[command(name = "prepkit-cli", version, about = "Asset-prep tools: fix sprite edges and normalize audio.")]
struct Cli {
    #[command(subcommand)]
    command: Command,

    /// Print one JSON summary instead of a line per file.
    #[arg(long, global = true)]
    json: bool,
}

#[derive(Subcommand)]
enum Command {
    /// Fill fully transparent pixels with the nearest edge color.
    PixelFix {
        /// Image files or folders.
        #[arg(required = true)]
        paths: Vec<PathBuf>,
        /// Include sub-folders.
        #[arg(short, long)]
        recursive: bool,
        /// Where results go: over the originals, into a sub-folder, or next to them with a prefix.
        #[arg(long, value_enum, default_value_t = CliOutput::Replace)]
        output: CliOutput,
        /// Sub-folder name for `--output folder`.
        #[arg(long, default_value = "fixed")]
        folder: String,
        /// File name prefix for `--output prefix`.
        #[arg(long, default_value = "fixed_")]
        prefix: String,
        /// Make filled pixels opaque, to see what was filled.
        #[arg(long, hide = true)]
        debug: bool,
    },
    /// Normalize and/or convert audio into a `normalized/` folder next to each file. Needs ffmpeg.
    #[command(alias = "audio-normalize")]
    AudioConvert {
        /// Audio files or folders (folders are read one level deep).
        #[arg(required = true)]
        paths: Vec<PathBuf>,
        /// RMS: average loudness hits the target, peaks capped at -1 dBFS. Peak: the loudest sample hits it.
        #[arg(long, value_enum, default_value_t = CliMode::Rms)]
        mode: CliMode,
        /// Target in dBFS [default: -14 for rms, -1 for peak].
        #[arg(long, allow_hyphen_values = true)]
        target: Option<f64>,
        /// Only convert, without changing the volume.
        #[arg(long)]
        no_normalize: bool,
        /// Convert to this format instead of keeping each file's own.
        #[arg(long, value_enum)]
        convert: Option<Format>,
        /// Added to output file names.
        #[arg(long, default_value = "_n")]
        suffix: String,
        /// Replace outputs that already exist instead of skipping them.
        #[arg(long)]
        overwrite: bool,
    },
    /// Download ffmpeg to %LOCALAPPDATA%\Programs\ffmpeg and add it to your user PATH (Windows).
    InstallFfmpeg {
        /// Install somewhere else.
        #[arg(long)]
        dir: Option<PathBuf>,
        /// Don't touch PATH.
        #[arg(long)]
        no_path: bool,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum CliMode {
    Peak,
    Rms,
}

#[derive(Clone, Copy, ValueEnum)]
enum CliOutput {
    Replace,
    Folder,
    Prefix,
}

#[derive(Clone, Copy, ValueEnum)]
enum Format {
    Ogg,
    Mp3,
    Wav,
    Flac,
    M4a,
    Aac,
    Opus,
    Aiff,
    Wma,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let cancel = AtomicBool::new(false);
    match cli.command {
        Command::PixelFix { paths, recursive, output, folder, prefix, debug } => {
            let output = match output {
                CliOutput::Replace => Output::Replace,
                CliOutput::Folder => Output::Folder,
                CliOutput::Prefix => Output::Prefix,
            };
            let options = PixelFixOptions { recursive, output, folder, prefix, debug };
            let summary = pixel_fix::run(&paths, &options, &cancel, |result, _, _| report::file(cli.json, result));
            report::summary(cli.json, "pixel-fix", &summary)
        }
        Command::AudioConvert { paths, mode, target, no_normalize, convert, suffix, overwrite } => {
            let Some(found) = ffmpeg::locate() else {
                return report::error(
                    cli.json,
                    "ffmpeg wasn't found. Install it with `prepkit-cli install-ffmpeg` (adds it to your user PATH) \
                     or from https://ffmpeg.org, then run this again.",
                );
            };
            let mut options = AudioOptions {
                normalize: !no_normalize,
                mode: match mode {
                    CliMode::Peak => Mode::Peak,
                    CliMode::Rms => Mode::Rms,
                },
                convert: convert.map(|format| format.to_possible_value().unwrap().get_name().to_string()),
                suffix,
                overwrite,
                ..AudioOptions::default()
            };
            if let Some(target) = target {
                options.target_peak = target;
                options.target_rms = target;
            }
            let summary = audio::run(&paths, &options, &found.path, &cancel, |result, _, _| {
                report::file(cli.json, result)
            });
            report::summary(cli.json, "audio-convert", &summary)
        }
        Command::InstallFfmpeg { dir, no_path } => {
            let options = InstallOptions { dir: dir.unwrap_or_else(ffmpeg::install_dir), add_to_path: !no_path };
            report::install(cli.json, ffmpeg::install(&options, |progress| report::install_progress(cli.json, &progress)))
        }
    }
}
