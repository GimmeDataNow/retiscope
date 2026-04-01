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

hops = 0 means it is on the same local hub (hearing your own announce)
hubs do not count as hops themselves if there a local node is trying to connect to somewhere else.
