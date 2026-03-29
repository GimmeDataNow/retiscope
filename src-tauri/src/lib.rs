// use log::info;
use std::env;
use std::process::Stdio;

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;

use tauri::{AppHandle, Emitter, State};
use tracing::{debug, error, info, instrument, trace, warn};

mod cli;

struct ProcessState(Mutex<Option<Child>>);

#[tauri::command]
async fn start_logging_process(
    app: AppHandle,
    state: State<'_, ProcessState>,
    command_options: String,
) -> Result<(), String> {
    info!("Starting command");
    // kill existing process if one is already running
    let mut lock = state.0.lock().await;
    if let Some(mut old_child) = lock.take() {
        let _ = old_child.kill().await;
    }

    let current_exe =
        env::current_exe().map_err(|e| format!("Failed to find current executable: {}", e))?;

    let command_options = command_options.split_whitespace();
    // spawn the new process
    let mut child = Command::new(current_exe)
        .args(command_options)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;

    let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;

    // store the child handle so we can kill it later
    *lock = Some(child);
    drop(lock);

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
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(ProcessState(Mutex::new(None)))
        .invoke_handler(tauri::generate_handler![
            start_logging_process,
            stop_logging_process
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
