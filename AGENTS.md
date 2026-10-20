# AGENTS.md — PrepKit

Guide for AI agents (Claude Code, Cursor, Codex…). There are two ways to use this file:
- **Using PrepKit from another project** to prepare assets: read the first section only.
- **Working on PrepKit itself:** read the rest.

## Using PrepKit from another project

Run the launcher in this folder from anywhere. It finds or builds the CLI, or downloads it, on its own.

```bash
<prepkit folder>\prepkit.cmd pixel-fix <image file or folder> [--recursive] [--output replace|folder|prefix] --json
<prepkit folder>\prepkit.cmd audio-convert <audio file or folder> [--convert ogg] [--mode rms|peak] [--target -14] --json
```

- **Pixel Fix overwrites images by default** (`--output replace`). Run it on exported, game-ready images (PNG with
  transparency), not on source files, or keep the originals with `--output folder` (writes `fixed/<name>`) or
  `--output prefix` (writes `fixed_<name>`).
- **Audio Convert never touches the originals.** It normalizes to RMS −14 dBFS by default (peaks capped at −1).
  Outputs go to a `normalized/` folder next to each input, named `<name>_n.<ext>`. Existing outputs are skipped unless
  you pass `--overwrite`. (`audio-normalize` still works as an old name.)
- **Results:** `--json` prints one object, `{ tool, ok, summary: { total, written, skipped, failed, results[] } }`,
  where each result has `path`, `output`, `status`, `message`. The exit code is `1` if anything failed or nothing
  matched.
- **ffmpeg:** audio needs it. If the CLI says ffmpeg is missing, **ask the user** before running
  `prepkit.cmd install-ffmpeg`, because it downloads ffmpeg and adds it to their PATH.
- `prepkit.cmd --help` and `prepkit.cmd <command> --help` list every option.

## What this is

PrepKit is a Windows toolkit for preparing game assets. It's public (github.com/nyaridev/prepkit, MIT). One Rust core
does the work; the desktop app (Tauri 2 + React) and the CLI are thin front ends over it. Decisions:
`docs/adr/0001-architecture.md`.

## Layout

| Path | What |
|---|---|
| `crates/prepkit-core/` | All tool logic: `pixel_fix/` (fill + EDT), `audio/` (probe, export args, normalize), `ffmpeg/` (locate, install, user PATH), `batch.rs` (shared results, parallel runner, file discovery) |
| `crates/prepkit-cli/` | `prepkit-cli` (clap): `main.rs` (commands), `report.rs` (output) |
| `src-tauri/` | The app shell: `commands.rs` (tools + progress events), `files.rs` (expands inputs into file lists; opens listed images to the asset protocol for previews), `settings.rs` (settings JSON), `jobs.rs` (cancel flags), `tauri.conf.json` (borderless window), `capabilities/` |
| `src/` | The UI: `app/` (App, TitleBar, Tabs, hooks), `features/<tool>/` (one folder per tab), `components/` (FilePanel, Thumb, RunPanel, controls), `lib/` (backend, settings, types), `styles/` (design tokens) |
| `assets/brand/` | The Pip mark, favicon, app icon source, UI icons (`icons/`), README banners, `preview.html`; `propositions/` holds the explorations |
| `prepkit.cmd` | Launcher for scripts and agents |
| `build.bat`, `start-dev.bat` | Release build / dev window |

## Commands

| Command | What |
|---|---|
| `cargo test --workspace` | Rust tests |
| `pnpm check` | `tsc` + Biome + Vitest |
| `pnpm dev` | The UI alone in a browser at http://localhost:4390, with a mock backend (`?ffmpeg=missing` previews the download banner) |
| `start-dev.bat` | The real app with hot reload |
| `build.bat` | `release\PrepKit.exe` + `release\prepkit-cli.exe` |

## Rules

1. **Logic lives in `prepkit-core`.** The app and the CLI only parse input, call the core and show results. Never
   process files in the UI or in `src-tauri`.
2. **The CLI's defaults equal the app's defaults** (`AudioOptions::default()`, `DEFAULT_SETTINGS` in
   `src/lib/settings.ts`). The CLI never reads the app's saved settings.
3. **Settings never contain folders or files.** The frontend owns the settings shape and repairs anything invalid in
   `mergeSettings`.
4. **Dark theme only:**
   - deep ink surfaces with faint steel/silver/gold glows, grain and a vignette;
   - a steel-grey accent (`#a9b4c8`) with dark text on it, muted gold ornaments;
   - the brand mark is Pip (`assets/brand/`); the app icon and title-bar logo use the accent;
   - Nunito Sans with a Cinzel title;
   - a transparent window that draws its own rounded body.

   Every color comes from `src/styles/tokens.css`. There's no settings screen; options sit in each tab's compact
   options card.
5. **The asset protocol scope stays empty in config.** Only `list_images` opens files to it, one by one, for previews.
6. **ffmpeg is never bundled.** PrepKit uses the system ffmpeg, or installs it on the user's request.
7. **Windows first.** Spawn ffmpeg through `ffmpeg::command()` (no console windows). Use `.bat` with CRLF.
8. Keep files small and focused. Each tool gets its own core module, CLI subcommand and UI folder.

## Adding a tool

1. A module in `crates/prepkit-core/src/<tool>/` with an options struct (`Default` = the defaults), `discover`, and
   `run(paths, options, cancel, on_result) -> Summary` built on `batch::run_parallel`. Add tests.
2. A subcommand in `crates/prepkit-cli/src/main.rs`.
3. A command in `src-tauri/src/commands.rs` that emits `job-progress` with the tool's job id.
4. A tab: `src/features/<tool>/<Tool>Tab.tsx`, its settings in `src/lib/settings.ts` (plus a test), an entry in
   `src/app/Tabs.tsx`, and the job id in `src/lib/types.ts`.
5. Document it in `README.md` (Tools, Command line) and in the first section of this file.
