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

here is the issue: I have my database trait RetiscopeDB. Now i need to implement a way for me to get live updates from the tables “announces” and “nodes”. What I really need is for the surrealdb implementation of the trait to return a sort of database event that is completely agnostic to the database implementation, meaning that if i were to implement postgres or similar then it would have to return the same struct.

This is a record from the announce table from my surrealdb.
{
	destination: node:bc7cabf778c26165958f419f01aab272,
	hops: 8,
	id: announce:00jcxv9v6i6xyfkg6lxu,
	iface: '7c9fa136d4413fa6173637e883b6998d',
	timestamp: d'2026-04-09T09:39:40.388173799Z',
	transport_node: node:7cbbe5ada62d88ee2d4dbe0c3cb1bceb
}
