@AGENTS.md

## Claude Code notes

- To check UI changes, run `pnpm dev` and open http://localhost:4390 in the browser pane at 920×660 (the window's
  default size). The mock backend simulates runs. Check both tabs, and `?ffmpeg=missing`.
- The real window (borderless title bar, drag and drop, settings file) only exists in `start-dev.bat` or a built
  `target\release\prepkit.exe`.
- Test audio and images belong in the scratchpad. Generate audio with `ffmpeg -f lavfi -i "sine=frequency=440:duration=1"`.
- Never run `install-ffmpeg` without `--no-path` and a scratchpad `--dir`, unless the user asks: it edits their PATH.
- Finish with `cargo test --workspace` and `pnpm check`.
