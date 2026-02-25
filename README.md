# Tauri + Solid

This template should help get you started developing with Tauri and Solid in Vite.

## Libraries and other utilites

- Node visualization - @dschz/solid-g6

## Output of the setup

Your system is missing dependencies (or they do not exist in $PATH):
╭────────────────────┬─────────────────────────────────────────────────────╮
│ Bun                │ Visit https://bun.sh/                               │
├────────────────────┼─────────────────────────────────────────────────────┤
│ webkit2gtk & rsvg2 │ Visit https://tauri.app/guides/prerequisites/#linux │
╰────────────────────┴─────────────────────────────────────────────────────╯

Make sure you have installed the prerequisites for your OS: https://tauri.app/start/prerequisites/, then run:
  cd retiscope
  bun install
  bun run tauri android init

For Desktop development, run:
  bun run tauri dev

For Android development, run:
  bun run tauri android dev
