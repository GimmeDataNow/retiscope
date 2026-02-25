use std::process::Stdio;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

#[tauri::command]
async fn start_logging_process(app: AppHandle) -> Result<(), String> {
    // 1. Spawn the child process
    let mut child = Command::new("ping") // Replace with your command
        .arg("google.com")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;

    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout).lines();

    // 2. Spawn an async task to stream logs
    tauri::async_runtime::spawn(async move {
        while let Ok(Some(line)) = reader.next_line().await {
            // 3. Emit the log line to the frontend
            app.emit("process-log", line).unwrap();
        }
    });

    Ok(())
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
        .invoke_handler(tauri::generate_handler![start_logging_process])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
