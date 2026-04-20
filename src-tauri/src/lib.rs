#[allow(unused_imports)]
use tracing::{debug, error, info, instrument, trace, warn};

use std::sync::Arc;

pub mod cli;
pub mod data;
pub mod errors;
pub mod files;
pub mod gui;

use crate::data::database::{self, RetiscopeDB};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() {
    info!("Listener started");
    // database
    let db_config_path = files::get_paths().config.join("database.toml");
    let db_config = database::load_database_config(db_config_path);
    // let db: Arc<dyn RetiscopeDB + Send + Sync> = db_config.create_db().await.unwrap();
    let db: Arc<dyn RetiscopeDB + Send + Sync> = db_config.create_db().await.unwrap();
    // this is needed
    let _ = db.init_db().await;

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(gui::DbWrapper { db })
        .invoke_handler(tauri::generate_handler![
            gui::start_db_watch,
            gui::fetch_announces_db,
            gui::fetch_nodes_db
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
