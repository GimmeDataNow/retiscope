use log::info;
use serde::{Deserialize, Serialize};
use std::env;
use std::process::Stdio;

use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::{self, Surreal};
use surrealdb_types::{RecordId, SurrealValue};

use tauri::{AppHandle, Emitter, State};

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;

mod cli;

#[tauri::command]
async fn get_graph_data() -> Result<Vec<cli::PathEntryWrapper>, String> {
    // 1. Connect (Consider keeping the DB handle in Tauri State instead of re-connecting every time)
    let db = Surreal::new::<Ws>("127.0.0.1:8000")
        .await
        .map_err(|e| e.to_string())?;

    db.signin(Root {
        username: "a".into(),
        password: "a".into(),
    })
    .await
    .map_err(|e| e.to_string())?;

    db.use_ns("main")
        .use_db("main")
        .await
        .map_err(|e| e.to_string())?;

    // 2. Query and Map
    let mut response = db
        .query("SELECT * FROM path_table;")
        .await
        .map_err(|e| e.to_string())?;

    // .take(0) pulls the result from the first statement in the query string
    let rows: Vec<cli::PathEntryWrapper> = response.take(0).map_err(|e| e.to_string())?;
    // log::trace!("{:?}", rows);

    Ok(rows)
}

#[derive(Debug, Serialize, Deserialize, SurrealValue)]
pub struct GatewaySummary {
    gateway_address: String,
    nodes_reachable: u64,
    min_hops: u8,
    primary_iface: Vec<String>,
}

#[tauri::command]
async fn get_gateway_summary() -> Result<Vec<GatewaySummary>, String> {
    // 1. Connect (Consider keeping the DB handle in Tauri State instead of re-connecting every time)
    let db = Surreal::new::<Ws>("127.0.0.1:8000")
        .await
        .map_err(|e| e.to_string())?;

    db.signin(Root {
        username: "a".into(),
        password: "a".into(),
    })
    .await
    .map_err(|e| e.to_string())?;

    db.use_ns("main")
        .use_db("main")
        .await
        .map_err(|e| e.to_string())?;

    // 2. The Query
    // Note: We use string::slice to handle the 'path_table:' prefix if present
    let sql = "
        SELECT 
            received_from AS gateway_address,
            count() AS nodes_reachable,
            math::min(hops) AS min_hops,
            array::distinct(iface) AS primary_iface
        FROM path_table 
        WHERE 
            received_from != string::split(meta::tb(id), ':')[1]
        GROUP BY 
            received_from;
    ";

    let mut response = db.query(sql).await.map_err(|e| e.to_string())?;

    // 3. Extract results
    let summaries: Vec<GatewaySummary> = response.take(0).map_err(|e| e.to_string())?;

    Ok(summaries)
}

// A struct to hold our active process
struct ProcessState(Mutex<Option<Child>>);

#[tauri::command]
async fn start_logging_process(
    app: AppHandle,
    state: State<'_, ProcessState>,
    command_options: String,
) -> Result<(), String> {
    info!("Starting command");
    // 1. Kill existing process if one is already running
    let mut lock = state.0.lock().await;
    if let Some(mut old_child) = lock.take() {
        let _ = old_child.kill().await;
    }

    let current_exe =
        env::current_exe().map_err(|e| format!("Failed to find current executable: {}", e))?;

    let command_options = command_options.split_whitespace();
    // 2. Spawn the new process
    let mut child = Command::new(current_exe)
        .args(command_options)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;

    let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;

    // 3. Store the child handle so we can kill it later
    *lock = Some(child);
    drop(lock); // Release lock so other commands can use it

    // 4. Stream logs in the background
    tauri::async_runtime::spawn(async move {
        let mut reader = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            let _ = app.emit("process-log", line);
        }
    });

    Ok(())
}

#[tauri::command]
async fn stop_logging_process(state: State<'_, ProcessState>) -> Result<(), String> {
    info!("Stopped command");
    let mut lock = state.0.lock().await;
    if let Some(mut child) = lock.take() {
        child.kill().await.map_err(|e| e.to_string())?;
        return Ok(());
    }
    Err("No process was running".to_string())
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(ProcessState(Mutex::new(None)))
        .invoke_handler(tauri::generate_handler![
            get_graph_data,
            get_gateway_summary,
            start_logging_process,
            stop_logging_process
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
