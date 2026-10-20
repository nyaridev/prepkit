<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="assets/brand/banner-dark.svg" />
    <img src="assets/brand/banner-light.svg" width="100%" alt="PrepKit — get game assets ready in one drop" />
  </picture>
</p>

<p align="center">
  <a href="https://github.com/nyaridev/prepkit/releases/latest/download/PrepKit.exe"><b>Download</b></a> ·
  <a href="https://github.com/nyaridev/prepkit/releases">Releases</a> ·
  <a href="#cli">CLI</a> ·
  <a href="AGENTS.md">Agents</a> ·
  <a href="LICENSE">MIT</a>
</p>

Drop files or folders, press one button. Windows 10/11, ~13 MB, no installer.

- **Pixel Fix** fills transparent pixels with the nearest edge color, so sprites don't fringe when scaled. Overwrites
  by default, or writes to `fixed/` or `fixed_<name>`.
- **Audio Convert** normalizes loudness (RMS −14 dBFS or a peak target) into `normalized/` and can convert the format.
  Needs [FFmpeg](https://ffmpeg.org), which PrepKit can download for you.

## CLI

`prepkit-cli.exe` is on the [Releases](https://github.com/nyaridev/prepkit/releases) page. AI agents can run
`prepkit.cmd` from this folder instead; it finds, builds or downloads the CLI ([AGENTS.md](AGENTS.md)).

```bash
prepkit-cli pixel-fix <path> --recursive --output folder
prepkit-cli audio-convert <path> --convert ogg
prepkit-cli --help
```

`--json` prints one summary. Exit code `1` means something failed.

## Build

Node 22+, pnpm and Rust. `start-dev.bat` runs the app, `build.bat` builds `release\PrepKit.exe` and
`release\prepkit-cli.exe`, and `cargo test --workspace` + `pnpm check` test it.

<sub>Audio by [FFmpeg](https://ffmpeg.org), run but not shipped.</sub>
