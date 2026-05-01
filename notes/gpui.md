```rust
struct AppSettings {
    pub theme_name: String,
    pub buffer_size: usize,
}
impl Global for AppSettings {}

// In main.rs
let settings = load_settings_from_file("config.toml");
cx.set_global(settings);
```
```rust
cx.on_next_frame()
```

retiscope/
├── assets/              # Icons, fonts, themes
├── src/
│   ├── core/            # Pure data types (Nodes, Packets, Links)
│   │   ├── mod.rs
│   │   └── types.rs     # Shared structs across the whole app
│   ├── db/              # Database Abstraction Layer (DAL)
│   │   ├── mod.rs       # The Repository trait
│   │   ├── sqlite.rs    # Implementation 1
│   │   └── postgres.rs  # Implementation 2
│   ├── network/         # Reticulum-specific logic
│   │   ├── processor.rs # High-speed packet handling
│   │   └── listener.rs  # Background thread management
│   ├── ui/              # GPUI Views and Components
│   │   ├── components/  # Reusable UI bits (Sidebar, Cards)
│   │   ├── pages/       # Dashboard, Settings, MapView
│   │   └── mod.rs       # AppView and Routing
│   ├── state.rs         # GPUI Globals and Models (The "Glue")
│   ├── paths.rs         # Path management (as discussed)
│   ├── error.rs         # Custom Error enum (using 'thiserror')
│   └── main.rs          # Entry point and wiring
└── Cargo.toml

src/
├── ui/
│   ├── mod.rs          # UI entry point & PageId enum
│   ├── app_view.rs     # The main shell (Sidebar + Header)
│   ├── components/     # Shared UI widgets (cards, specialized buttons)
│   └── pages/          # Individual feature views
│       ├── mod.rs
│       ├── dashboard.rs
│       ├── settings.rs
│       └── network_graph.rs
