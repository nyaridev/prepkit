//! ffmpeg encoder arguments for an output format, keeping the source's sample rate, channels and bit depth.
//! A port of audio-normalize's `build_export_kwargs`, plus three fixes:
//! - opus keeps libopus's own sample rate (it can't take 44.1 kHz);
//! - FLAC from a float source uses `s32`;
//! - 8-bit FLAC uses `s16` (FLAC has no float or 8-bit sample format in ffmpeg).

use super::Probe;

const LOSSY: &[&str] = &["mp3", "ogg", "aac", "m4a", "wma", "opus"];
const DEFAULT_KBPS: u32 = 192;

/// Arguments to place between the input options and the output path.
pub fn export_args(extension: &str, probe: &Probe) -> Vec<String> {
    let extension = extension.to_ascii_lowercase();
    let mut args: Vec<String> = Vec::new();
    let mut push = |items: &[&str]| args.extend(items.iter().map(|item| item.to_string()));

    if let Some(rate) = probe.sample_rate.filter(|_| extension != "opus") {
        push(&["-ar", &rate.to_string()]);
    }
    if let Some(channels) = probe.channels {
        push(&["-ac", &channels.to_string()]);
    }

    if LOSSY.contains(&extension.as_str()) {
        match extension.as_str() {
            "ogg" => push(&["-c:a", "libvorbis"]),
            "opus" => push(&["-c:a", "libopus"]),
            "mp3" => push(&["-c:a", "libmp3lame"]),
            _ => {}
        }
        // Never reuse a PCM container's bitrate (often over 1000k) for a lossy encoder.
        let kbps = probe.bit_rate_kbps.filter(|k| (32..=500).contains(k)).unwrap_or(DEFAULT_KBPS);
        push(&["-b:a", &format!("{kbps}k")]);
    } else if extension == "wav" || extension == "aiff" || extension == "aif" {
        let codec = pcm_codec(&extension, probe);
        push(&["-c:a", codec]);
    } else if extension == "flac" {
        push(&["-sample_fmt", flac_sample_fmt(probe)]);
    }

    push(&["-f", muxer(&extension)]);
    args
}

fn muxer(extension: &str) -> &str {
    match extension {
        "aac" => "adts",
        "m4a" => "ipod",
        "wma" => "asf",
        "aif" => "aiff",
        other => other,
    }
}

fn is_float(probe: &Probe) -> bool {
    probe.sample_fmt.contains("flt") || probe.sample_fmt.contains("dbl")
}

/// The source's bit depth, guessed from the codec and sample format when ffmpeg didn't say.
fn depth(probe: &Probe) -> u32 {
    if let Some(bits) = probe.bit_depth {
        return bits;
    }
    let (codec, format) = (probe.codec.as_str(), probe.sample_fmt.as_str());
    if codec.contains("24") || format.contains("s24") {
        24
    } else if codec.contains("32") || format.contains("s32") || is_float(probe) {
        32
    } else if codec.contains('8') {
        8
    } else {
        16
    }
}

fn pcm_codec(extension: &str, probe: &Probe) -> &'static str {
    let little_endian = extension == "wav";
    if little_endian && is_float(probe) {
        return "pcm_f32le";
    }
    match (depth(probe), little_endian) {
        (8, _) => "pcm_u8",
        (24, true) => "pcm_s24le",
        (32, true) => "pcm_s32le",
        (24, false) => "pcm_s24be",
        (32, false) => "pcm_s32be",
        (_, true) => "pcm_s16le",
        (_, false) => "pcm_s16be",
    }
}

fn flac_sample_fmt(probe: &Probe) -> &'static str {
    if is_float(probe) {
        return "s32";
    }
    match depth(probe) {
        24 | 32 => "s32",
        _ => "s16",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn probe(codec: &str, sample_fmt: &str, bit_depth: Option<u32>, kbps: Option<u32>) -> Probe {
        Probe {
            codec: codec.into(),
            sample_rate: Some(44100),
            channels: Some(2),
            sample_fmt: sample_fmt.into(),
            bit_depth,
            bit_rate_kbps: kbps,
            ..Probe::default()
        }
    }

    fn joined(extension: &str, probe: &Probe) -> String {
        export_args(extension, probe).join(" ")
    }

    #[test]
    fn wav_to_ogg_uses_vorbis_at_192k() {
        let wav = probe("pcm_s16le", "s16", Some(16), Some(1411));
        assert_eq!(joined("ogg", &wav), "-ar 44100 -ac 2 -c:a libvorbis -b:a 192k -f ogg");
    }

    #[test]
    fn mp3_keeps_a_sane_source_bitrate() {
        let mp3 = probe("mp3", "fltp", None, Some(128));
        assert_eq!(joined("mp3", &mp3), "-ar 44100 -ac 2 -c:a libmp3lame -b:a 128k -f mp3");
    }

    #[test]
    fn wav_keeps_the_bit_depth() {
        assert!(joined("wav", &probe("pcm_s24le", "s32", Some(24), None)).contains("pcm_s24le"));
        assert!(joined("wav", &probe("mp3", "fltp", None, None)).contains("pcm_f32le"));
        assert!(joined("wav", &probe("pcm_u8", "u8", Some(8), None)).contains("pcm_u8"));
        assert!(joined("aiff", &probe("pcm_s16le", "s16", Some(16), None)).contains("pcm_s16be -f aiff"));
    }

    #[test]
    fn container_names_follow_the_original_map() {
        let source = probe("pcm_s16le", "s16", Some(16), None);
        assert!(joined("m4a", &source).ends_with("-f ipod"));
        assert!(joined("aac", &source).ends_with("-f adts"));
        assert!(joined("wma", &source).ends_with("-f asf"));
    }

    #[test]
    fn fixes_over_the_original() {
        let source = probe("pcm_s16le", "s16", Some(16), None);
        assert!(!joined("opus", &source).contains("-ar"), "libopus picks its own rate");
        assert!(joined("flac", &probe("mp3", "fltp", None, None)).contains("-sample_fmt s32"));
        assert!(joined("flac", &probe("pcm_u8", "u8", Some(8), None)).contains("-sample_fmt s16"));
    }
}
