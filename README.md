# Retiscope

Retiscope is a reticulum network visualizer.

Retiscope supports (or will support) multi node data retreival for viewing large networks. 

## Documentation
- [Protocol Specification](notes/PROTOCOL_SPECIFICATION.md) — How nodes talk to each other.
- [Architecture Overview](notes/ARCHITECTURE.md) — CLI vs. GUI vs. DB logic.

## Features / Planned Features

### General overview

- [ ] GUI with server and router
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
- [ ] Save known network map into a database

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
- [ ] Clustering
- [ ] Filtering
- [ ] Scope zooming
- [ ] Observed vs inferred distinction, Confidence weighting
- [ ] Path trace mode (Select node A → node B, Show inferred path + metrics.)
- [ ] Partition Detection (might be difficult)

### Telemetry Model
- Observed link (direct link established)
- Inferred route (via routing table)
- Announced presence (identity seen)


## Libraries and other utilites

- Reticulum - https://github.com/BeechatNetworkSystemsLtd/Reticulum-rs.git
- Node visualization - @dschz/solid-g6

# Development

Ensure a database is running using
```
surreal start --user a --pass a --bind 0.0.0.0:8000 rocksdb:$HOME/.local/share/retiscope/surreal/
```
> [!WARNING]
> Depending on the version the application may immideately crash if it fails to establish a connection. 

For running the listener daemon:
```
cd src-tauri/
cargo run -- daemon
```

For running the GUI:
```
bun run tauri dev
```


