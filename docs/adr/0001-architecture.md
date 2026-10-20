# ADR-0001: One Rust core behind a Tauri app and a CLI

**Status:** Accepted
**Date:** 2026-10-20
**Deciders:** nyaridev

## Context

Two separate Python tools prepared game assets:
- **pixel-fix:** fills transparent pixels with the nearest edge color. Pillow, NumPy and SciPy, with a pywebview GUI,
  shipped as a ~57 MB PyInstaller exe.
- **audio-normalize:** peak/RMS normalization plus conversion through pydub and ffmpeg, with a Tkinter GUI.

They're merging into one public toolkit, PrepKit, that will grow more tools. The requirements:

- **App:** a single, light `.exe` for other people, with a borderless dark window, a custom title bar, tabs, and no
  settings screen. Options are remembered between runs, but folders aren't.
- **Agents:** the same tools usable by AI agents in a cloned repo, with machine-readable output.
- **Audio formats:** every format the old audio tool handled (wav, ogg, mp3, flac, m4a, aac, opus, aiff, wma).
- **ffmpeg:** use the system one when it exists; offer to install it otherwise (download it and add it to the user's
  PATH).

## Decision

- **Structure:** a Cargo workspace with one library, `prepkit-core`, that holds all the logic. Two thin front ends sit
  on it:
  - `PrepKit.exe`: Tauri 2 with a React UI;
  - `prepkit-cli.exe`: clap, with `--json`.
- **Pixel Fix** runs natively: the `image` crate, an exact Euclidean distance transform with nearest-site indices
  (ported from pixel-fix's Python fallback), and rayon.
- **Audio** goes through the system ffmpeg:
  - One run measures (`volumedetect`) and reads stream info from stderr, so no ffprobe is needed.
  - A second run applies the gain, with encoder arguments ported from `build_export_kwargs`.
- **ffmpeg install:** if ffmpeg is missing, PrepKit downloads BtbN's build on request, verifies its SHA256, unpacks it
  to `%LOCALAPPDATA%\Programs\ffmpeg` and adds it to the user's PATH.
- **Settings:** a JSON object owned by the frontend, saved under the app's config folder. It never contains paths.
- **For agents:** a `prepkit.cmd` launcher in the repo finds, builds or downloads the CLI.

## Options considered

### Option A: Tauri 2 + Rust core (chosen)

| Dimension | Assessment |
|---|---|
| Complexity | Medium: the algorithms move from Python to Rust |
| Size and speed | ~13 MB app, ~6 MB CLI, instant start; native, parallel processing |
| Reach | WebView2 ships with Windows 10/11; the CLI builds on any OS |
| Familiarity | Matches nyaridev's stack (Tauri apps, Rust in ume-sync) |

**Pros:** small, fast, one core shared by both front ends, a modern UI.
**Cons:** rewriting the tools; Rust is needed to build from source.

### Option B: Python + pywebview + PyInstaller

**Pros:** reuses the existing code almost unchanged; agents can run it with `uv run`.
**Cons:** a 50–60 MB exe with a slow start; Python and NumPy have to live inside every release; a weaker window.

### Option C: Tauri UI + Python sidecar

**Pros:** keeps the Python logic and gets the Tauri window.
**Cons:** two runtimes, and the sidecar can't be a single exe; the worst of both.

### Option D: Bundle ffmpeg instead of using the system's

**Pros:** audio always works offline.
**Cons:** adds 80–190 MB to every download and ffmpeg license notices to the release. Rejected in favor of "use it if
it's there, install it on request".

## Trade-off analysis

**Size and speed vs. reuse.** The core of Pixel Fix is ~200 lines, and the audio logic is mostly ffmpeg arguments, so
porting them is cheap compared with shipping Python in every release.

**ffmpeg.** Keeping it a system dependency keeps the exe small and the license simple. The download button makes the
missing case one click, and installing it on PATH means other tools benefit too.

**One core.** The CLI and the app can't drift apart, because both call the same functions and share the same defaults.

## Consequences

- **Easier:**
  - adding tools (one core module, one subcommand, one tab);
  - agents using PrepKit (`prepkit.cmd … --json`);
  - shipping (two small exes).
- **Harder:** building from source needs Rust, Node and pnpm. Audio needs ffmpeg at run time.
- **To revisit:**
  - the agent path depends on a GitHub release existing (otherwise it needs Rust installed);
  - animated GIF/APNG keep only their first frame, as before;
  - lossy re-encodes drift peaks by about ±0.5 dB, as the old tool did.

## Action items

1. [x] Core: pixel fix (EDT + fill), audio (probe, export args, normalize), ffmpeg locate/install/PATH, tests.
2. [x] CLI with `--json`, and the `prepkit.cmd` launcher.
3. [x] Tauri app: borderless window, tabs, drag and drop, progress, cancel, saved settings, ffmpeg banner.
4. [ ] Publish the first GitHub release with `PrepKit.exe` and `prepkit-cli.exe`.
5. [ ] Archive the old `pixel-fix` and `audio-normalize` repos once PrepKit replaces them.
