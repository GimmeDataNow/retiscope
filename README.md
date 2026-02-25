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
- [ ] Edge confidence score
- [ ] Time decay
- [ ] Network map snapshots (both as json and as an image)

### Visualization

- [ ] Node clustering
- [ ] Transport grouping
- [ ] Edge wighting (quality/bandwith)
- [ ] Time decay (makes nodes fade)
- [ ] Layout stabilization
- [ ] Known identities
- [ ] Trust relationships
- [ ] Announce propagation (could be visually overbearing)
- [ ] Unknown nodes vs known nodes
- [ ] Get data from many servers
- [ ] Node aging
- [ ] clustering
- [ ] filtering
- [ ] scope zooming
- [ ] Observed vs inferred distinction, Confidence weighting

### Animations
- [ ] Hop sequence
- [ ] Time
- [ ] Retransmission (might be difficult)
- [ ] interface switches

### Telemetry Model
- Observed link (direct link established)
- Inferred route (via routing table)
- Announced presence (identity seen)


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
