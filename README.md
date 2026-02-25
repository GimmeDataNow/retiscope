# Retiscope

Retiscope is a reticulum network visualizer.

Retiscope supports (or will support) multi node data retreival for viewing large networks. 

## Features / Planned Features

### General overview

- [ ] Live topology graph
- [ ] Route table visualization
- [ ] Per-hop metrics
- [ ] Packet tracing
- [ ] Interface state
- [ ] MTU / bandwith / RSSI display
- [ ] Identity resolution mapping 
- [ ] Node distance + relay count

## Libraries and other utilites

- Reticulum - https://github.com/BeechatNetworkSystemsLtd/Reticulum-rs.git
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
