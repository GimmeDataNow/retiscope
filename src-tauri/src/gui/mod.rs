#[allow(unused_imports)]
use tracing::{debug, error, info, instrument, trace, warn};

use std::sync::Arc;
use tauri::Emitter;
use tauri::{AppHandle, Runtime, State};

use crate::data::database::RetiscopeDB;
use crate::data::{StoredAnnounce, StoredNode};

#[derive(Clone)]
pub struct DbWrapper {
    pub db: Arc<dyn RetiscopeDB + Send + Sync>, // the same type you create in listener/mod.rs
}

#[tauri::command]
pub async fn start_db_watch<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, DbWrapper>,
) -> Result<(), String> {
    let db = state.db.clone();

    let mut ann_rx = db.watch_announces().await.expect("watch_announces failed");
    let app_clone = app.clone();
    tokio::spawn(async move {
        while let Ok(ann) = ann_rx.recv().await {
            app_clone.emit("announce_changed", ann).ok();
        }
    });

    let mut node_rx = db.watch_nodes().await.expect("watch_nodes failed");
    tokio::spawn(async move {
        while let Ok(node) = node_rx.recv().await {
            app.emit("node_changed", node).ok();
        }
    });
    Ok(())
}

#[tauri::command]
pub async fn fetch_announces_db<R: Runtime>(
    _app: AppHandle<R>,
    state: State<'_, DbWrapper>,
) -> Result<Vec<StoredAnnounce>, String> {
    let data = state.db
        .fetch_announces()
        .await
        .inspect_err(|e| error!(error = %e, "Failed to fetch announces"))
        .expect("fetch_annouces_db failed");

    Ok(data)
}

#[tauri::command]
pub async fn fetch_nodes_db<R: Runtime>(
    _app: AppHandle<R>,
    state: State<'_, DbWrapper>,
) -> Result<Vec<StoredNode>, String> {
    let data = state.db
        .fetch_nodes()
        .await
        .expect("fetch_nodes_db failed");

    Ok(data)
}
