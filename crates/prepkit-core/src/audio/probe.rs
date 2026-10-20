//! Reads a file's stream info and levels from one ffmpeg run: `ffmpeg -i <file> -af volumedetect -f null -`.
//! ffmpeg prints both to stderr, so no ffprobe is needed.

use std::path::Path;
use std::sync::LazyLock;

use anyhow::{bail, Context, Result};
use regex::Regex;

use crate::ffmpeg;

/// What the encoder settings and the gain are based on.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Probe {
    pub codec: String,
    pub sample_rate: Option<u32>,
    pub channels: Option<u32>,
    /// ffmpeg's sample format, e.g. `s16`, `s32`, `fltp`.
    pub sample_fmt: String,
    pub bit_depth: Option<u32>,
    pub bit_rate_kbps: Option<u32>,
    /// RMS level in dBFS.
    pub mean_volume: Option<f64>,
    /// Peak level in dBFS.
    pub max_volume: Option<f64>,
}

static STREAM: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"Stream #\d+:\d+[^:]*: Audio: (.+)").unwrap());
static HERTZ: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\d+) Hz$").unwrap());
static KBPS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(\d+) kb/s").unwrap());
static BITS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\((\d+) bit\)").unwrap());
static CHANNELS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\d+) channels").unwrap());
static MEAN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"mean_volume: (-?[\d.]+|-?inf) dB").unwrap());
static MAX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"max_volume: (-?[\d.]+|-?inf) dB").unwrap());

/// Runs the measuring pass on `input`.
pub fn run(ffmpeg_path: &Path, input: &Path) -> Result<Probe> {
    let output = ffmpeg::command(ffmpeg_path)
        .args(["-hide_banner", "-nostdin", "-i"])
        .arg(input)
        .args(["-map", "0:a:0", "-af", "volumedetect", "-f", "null", "-"])
        .output()
        .context("Couldn't run ffmpeg")?;
    let stderr = String::from_utf8_lossy(&output.stderr);
    let probe = parse(&stderr);
    if !output.status.success() || probe.max_volume.is_none() {
        let reason = stderr.lines().rev().find(|line| !line.trim().is_empty()).unwrap_or("no audio stream");
        bail!("Couldn't read the audio: {}", reason.trim());
    }
    Ok(probe)
}

/// Parses ffmpeg's stderr. The first audio stream line is the input's; later ones describe outputs.
pub fn parse(stderr: &str) -> Probe {
    let mut probe = Probe {
        mean_volume: level(&MEAN, stderr),
        max_volume: level(&MAX, stderr),
        ..Probe::default()
    };
    let Some(stream) = STREAM.captures(stderr).map(|c| c[1].to_string()) else { return probe };

    let parts: Vec<&str> = stream.split(", ").map(str::trim).collect();
    probe.codec = parts[0].split_whitespace().next().unwrap_or_default().to_string();
    probe.bit_rate_kbps = parts.iter().find_map(|part| KBPS.captures(part)).and_then(|c| c[1].parse().ok());

    if let Some(hz) = parts.iter().position(|part| HERTZ.is_match(part)) {
        probe.sample_rate = HERTZ.captures(parts[hz]).and_then(|c| c[1].parse().ok());
        probe.channels = parts.get(hz + 1).and_then(|layout| channel_count(layout));
        if let Some(format) = parts.get(hz + 2) {
            probe.sample_fmt = format.split_whitespace().next().unwrap_or_default().to_string();
            probe.bit_depth = BITS.captures(format).and_then(|c| c[1].parse().ok()).or_else(|| depth_of(&probe.sample_fmt));
        }
    }
    probe
}

fn level(pattern: &Regex, stderr: &str) -> Option<f64> {
    let value = pattern.captures(stderr)?[1].to_string();
    if value.ends_with("inf") {
        return Some(f64::NEG_INFINITY);
    }
    value.parse().ok()
}

/// "stereo" → 2, "5.1(side)" → 6, "3 channels" → 3.
fn channel_count(layout: &str) -> Option<u32> {
    if let Some(captures) = CHANNELS.captures(layout) {
        return captures[1].parse().ok();
    }
    let name = layout.split('(').next().unwrap_or(layout);
    match name {
        "mono" => Some(1),
        "stereo" => Some(2),
        "2.1" | "3.0" => Some(3),
        "quad" | "4.0" | "3.1" => Some(4),
        "5.0" | "4.1" => Some(5),
        "5.1" | "6.0" => Some(6),
        "6.1" | "7.0" => Some(7),
        "7.1" => Some(8),
        _ => None,
    }
}

/// Bit depth implied by an integer sample format; float formats have none.
fn depth_of(sample_fmt: &str) -> Option<u32> {
    match sample_fmt.trim_end_matches('p') {
        "u8" => Some(8),
        "s16" => Some(16),
        "s32" => Some(32),
        "s64" => Some(64),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WAV_24: &str = "  Duration: 00:00:01.00, bitrate: 2304 kb/s
  Stream #0:0: Audio: pcm_s24le ([1][0][0][0] / 0x0001), 48000 Hz, stereo, s32 (24 bit), 2304 kb/s
  Stream #0:0 -> #0:0 (pcm_s24le (native) -> pcm_s16le (native))
  Stream #0:0: Audio: pcm_s16le, 48000 Hz, stereo, s16, 1536 kb/s
[Parsed_volumedetect_0 @ 00000221b002e0c0] mean_volume: -36.1 dB
[Parsed_volumedetect_0 @ 00000221b002e0c0] max_volume: -33.1 dB";

    const MP3: &str = "  Stream #0:0: Audio: mp3 (mp3float), 48000 Hz, stereo, fltp, 128 kb/s
  Stream #0:0: Audio: pcm_s16le, 48000 Hz, stereo, s16, 1536 kb/s
[Parsed_volumedetect_0 @ 000001949c69b3c0] mean_volume: -36.5 dB
[Parsed_volumedetect_0 @ 000001949c69b3c0] max_volume: -33.5 dB";

    const M4A: &str = "  Stream #0:0[0x1](und): Audio: aac (LC) (mp4a / 0x6134706D), 44100 Hz, mono, fltp, 96 kb/s (default)
[Parsed_volumedetect_0 @ 0] max_volume: -0.0 dB";

    #[test]
    fn reads_a_24_bit_wav_from_the_input_line() {
        let probe = parse(WAV_24);
        assert_eq!(probe.codec, "pcm_s24le");
        assert_eq!((probe.sample_rate, probe.channels), (Some(48000), Some(2)));
        assert_eq!((probe.sample_fmt.as_str(), probe.bit_depth), ("s32", Some(24)));
        assert_eq!(probe.bit_rate_kbps, Some(2304));
        assert_eq!((probe.mean_volume, probe.max_volume), (Some(-36.1), Some(-33.1)));
    }

    #[test]
    fn reads_lossy_streams() {
        let mp3 = parse(MP3);
        assert_eq!((mp3.codec.as_str(), mp3.sample_fmt.as_str()), ("mp3", "fltp"));
        assert_eq!((mp3.bit_depth, mp3.bit_rate_kbps), (None, Some(128)));

        let m4a = parse(M4A);
        assert_eq!((m4a.codec.as_str(), m4a.channels, m4a.sample_rate), ("aac", Some(1), Some(44100)));
        assert_eq!((m4a.bit_rate_kbps, m4a.max_volume, m4a.mean_volume), (Some(96), Some(-0.0), None));
    }

    #[test]
    fn handles_silence_and_missing_streams() {
        let probe = parse("mean_volume: -inf dB\nmax_volume: -inf dB");
        assert_eq!(probe.max_volume, Some(f64::NEG_INFINITY));
        assert!(probe.codec.is_empty());
    }
}
