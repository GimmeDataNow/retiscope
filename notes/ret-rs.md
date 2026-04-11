surreal start --user a --pass a --bind 0.0.0.0:8000 rocksdb:$HOME/.local/share/retiscope/surreal/

use tauri state to pass the connection from the backend to the frontend
would look something like this:
```rust
#[tokio::main]
async fn main() {
    // 1. Init DB
    let db = cli::db_init().await; 
    
    // 2. Start your background router (Pass a clone of DB to it)
    let db_clone = db.clone();
    tokio::spawn(async move {
        cli::router_init(db_clone).await;
    });

    tauri::Builder::default()
        // 3. Manage the DB handle so Commands can see it
        .manage(db) 
        .invoke_handler(tauri::generate_handler![get_announces])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```
```rust
use tauri::State;
use surrealdb::{Surreal, engine::remote::ws::Client};

#[tauri::command]
pub async fn get_announces(
    db: State<'_, Surreal<Client>>
) -> Result<Vec<AnnounceData>, String> {
    // Query the DB for the last 50 announces for the graph
    let mut response = db
        .query("SELECT * FROM path_table ORDER BY last_seen DESC LIMIT 50")
        .await
        .map_err(|e| e.to_string())?;

    let announces: Vec<AnnounceData> = response.take(0).map_err(|e| e.to_string())?;
    Ok(announces)
}
```
2026-04-09T23:29:04.283688Z ERROR no good error=Error("failed to parse", line: 0, column: 0) data=Object {"destination": String("node:234eed3d3775eb1e29cf5a3842961c25"), "hops": Number(0), "id": String("announce:f0mkch2i0e0a9odxagll"), "iface": String("7c9fa136d4413fa6173637e883b6998d"), "timestamp": String("2026-04-09T23:29:04.209327549Z"), "transport_node": String("node:NULL")}
Why does this error? Simple, the current impl kinda sucks.
transport_node can be node:NULL which is not a valid hex string. This probably comes from my bad impl of the serialize impl.
