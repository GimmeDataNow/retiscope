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

Now the packet view is working i have found a potential bug:
The local interface seems to be broadcasting a LOT. It seems a bit too frequent for my taste.
I might need a way for me to tweak that value
