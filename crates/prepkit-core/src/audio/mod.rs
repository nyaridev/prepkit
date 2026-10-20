//! Audio Normalize: brings every file to a peak or RMS target and writes it to a `normalized/` folder next to the
//! original, optionally in another format. ffmpeg does the decoding, the gain and the encoding.

mod export_args;
mod probe;

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

use crate::batch::{self, FileResult, Status, Summary};
use crate::ffmpeg;

pub use export_args::export_args;
pub use probe::{parse as parse_probe, Probe};

pub const AUDIO_EXTENSIONS: &[&str] = &["wav", "ogg", "mp3", "flac", "aac", "m4a", "wma", "aiff", "aif", "opus"];

/// The folder (next to each input) that outputs go into.
pub const OUTPUT_DIR: &str = "normalized";

/// Peaks never go above this in RMS mode.
const RMS_PEAK_CEILING: f64 = -1.0;

/// At or below this mean level a file counts as silent and isn't amplified.
const SILENCE_DB: f64 = -90.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    /// The loudest sample reaches the target. Safe for sound effects.
    Peak,
    /// The average loudness reaches the target, with peaks capped at -1 dBFS.
    Rms,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AudioOptions {
    pub normalize: bool,
    pub mode: Mode,
    /// dBFS target in peak mode.
    pub target_peak: f64,
    /// dBFS target in RMS mode.
    pub target_rms: f64,
    /// Output format (extension without the dot), or `None` to keep each file's format.
    pub convert: Option<String>,
    /// Added to each output's file name: `jump.wav` → `normalized/jump_n.wav`.
    pub suffix: String,
    /// Replace outputs that already exist instead of skipping them.
    pub overwrite: bool,
}

impl Default for AudioOptions {
    fn default() -> Self {
        Self {
            normalize: true,
            mode: Mode::Rms,
            target_peak: -1.0,
            target_rms: -14.0,
            convert: None,
            suffix: "_n".into(),
            overwrite: false,
        }
    }
}

/// The audio files that `paths` (files and folders, one level deep) point to.
pub fn discover(paths: &[PathBuf]) -> Vec<PathBuf> {
    batch::collect_files(paths, false, AUDIO_EXTENSIONS)
}

/// Processes every audio file under `paths` with the ffmpeg at `ffmpeg_path`, reporting each file as it finishes.
pub fn run<P>(paths: &[PathBuf], options: &AudioOptions, ffmpeg_path: &Path, cancel: &AtomicBool, on_result: P) -> Summary
where
    P: Fn(&FileResult, usize, usize) + Sync,
{
    let files = discover(paths);
    batch::run_parallel(files, cancel, |path| process_file(path, options, ffmpeg_path), on_result)
}

/// Where `input`'s output goes: `<dir>/normalized/<stem><suffix>.<ext>`.
pub fn output_path(input: &Path, options: &AudioOptions) -> PathBuf {
    let stem = input.file_stem().unwrap_or_default().to_string_lossy();
    let extension = match &options.convert {
        Some(format) => format.to_ascii_lowercase(),
        None => input.extension().unwrap_or_default().to_string_lossy().to_ascii_lowercase(),
    };
    let folder = input.parent().unwrap_or(Path::new(".")).join(OUTPUT_DIR);
    folder.join(format!("{stem}{}.{extension}", options.suffix))
}

/// The gain (dB) that brings a file with these levels to the target, or `None` when it should stay as it is.
pub fn gain_db(probe: &Probe, options: &AudioOptions) -> Option<f64> {
    if !options.normalize {
        return None;
    }
    let peak = probe.max_volume?;
    match options.mode {
        Mode::Peak => Some(options.target_peak - peak),
        Mode::Rms => {
            let mean = probe.mean_volume?;
            if mean <= SILENCE_DB {
                return None;
            }
            let gain = options.target_rms - mean;
            Some(if peak + gain > RMS_PEAK_CEILING { RMS_PEAK_CEILING - peak } else { gain })
        }
    }
}

/// Normalizes (and converts) one file.
pub fn process_file(input: &Path, options: &AudioOptions, ffmpeg_path: &Path) -> FileResult {
    let output = output_path(input, options);
    let exists = fs::metadata(&output).is_ok_and(|meta| meta.len() > 0);
    if exists && !options.overwrite {
        return FileResult::new(input, Status::Skipped, "Output already exists").with_output(output);
    }
    match encode(input, &output, options, ffmpeg_path) {
        Ok(Some(gain)) => FileResult::new(input, Status::Written, format!("{gain:+.1} dB")).with_output(output),
        Ok(None) => FileResult::new(input, Status::Written, "Copied without gain").with_output(output),
        Err(error) => FileResult::new(input, Status::Failed, format!("{error:#}")).with_output(output),
    }
}

/// Measures, then writes `output` through a `.part` file. Returns the gain that was applied.
fn encode(input: &Path, output: &Path, options: &AudioOptions, ffmpeg_path: &Path) -> Result<Option<f64>> {
    let probe = probe::run(ffmpeg_path, input)?;
    let gain = gain_db(&probe, options);
    let extension = output.extension().unwrap_or_default().to_string_lossy().to_string();

    fs::create_dir_all(output.parent().context("Output has no folder")?)?;
    let mut part = output.as_os_str().to_owned();
    part.push(".part");
    let part = PathBuf::from(part);

    let mut command = ffmpeg::command(ffmpeg_path);
    command.args(["-hide_banner", "-nostdin", "-loglevel", "error", "-y", "-i"]).arg(input).args(["-map", "0:a:0"]);
    if let Some(gain) = gain {
        command.arg("-af").arg(format!("volume={gain:.2}dB"));
    }
    command.args(export_args(&extension, &probe)).arg(&part);

    let result = command.output().context("Couldn't run ffmpeg")?;
    let written = fs::metadata(&part).map(|meta| meta.len()).unwrap_or(0);
    if !result.status.success() || written == 0 {
        let _ = fs::remove_file(&part);
        let stderr = String::from_utf8_lossy(&result.stderr);
        let reason = stderr.lines().rev().find(|line| !line.trim().is_empty()).unwrap_or("no output");
        bail!("ffmpeg failed: {}", reason.trim());
    }
    fs::rename(&part, output).context("Couldn't move the output into place")?;
    Ok(gain)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn levels(mean: f64, max: f64) -> Probe {
        Probe { mean_volume: Some(mean), max_volume: Some(max), ..Probe::default() }
    }

    #[test]
    fn peak_mode_moves_the_loudest_sample_to_the_target() {
        let options = AudioOptions { mode: Mode::Peak, ..AudioOptions::default() };
        let gain = gain_db(&levels(-30.0, -12.0), &options).unwrap();
        assert!((gain - 11.0).abs() < 1e-9);
    }

    #[test]
    fn rms_mode_caps_peaks_at_minus_one() {
        let options = AudioOptions::default();
        assert_eq!(options.mode, Mode::Rms, "RMS -14 is the default");
        let gain = gain_db(&levels(-30.0, -20.0), &options).unwrap();
        assert!((gain - 16.0).abs() < 1e-9, "-30 → -14 needs +16, peak ends at -4");

        let capped = gain_db(&levels(-30.0, -5.0), &options).unwrap();
        assert!((capped - 4.0).abs() < 1e-9, "a +16 gain would clip, so the peak stops at -1");
    }

    #[test]
    fn silence_and_disabled_normalize_apply_no_gain() {
        let rms = AudioOptions { mode: Mode::Rms, ..AudioOptions::default() };
        assert_eq!(gain_db(&levels(-91.0, -91.0), &rms), None);
        let off = AudioOptions { normalize: false, ..AudioOptions::default() };
        assert_eq!(gain_db(&levels(-30.0, -12.0), &off), None);
    }

    #[test]
    fn outputs_go_into_normalized_with_the_suffix() {
        let input = Path::new("sfx").join("Jump.WAV");
        let keep = output_path(&input, &AudioOptions::default());
        assert_eq!(keep, Path::new("sfx").join("normalized").join("Jump_n.wav"));

        let convert = AudioOptions { convert: Some("ogg".into()), ..AudioOptions::default() };
        assert_eq!(output_path(&input, &convert), Path::new("sfx").join("normalized").join("Jump_n.ogg"));
    }
}
