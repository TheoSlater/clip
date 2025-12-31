use std::sync::Mutex;

use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_shell::{
    process::{CommandChild, CommandEvent},
    ShellExt,
};

struct CaptureProcess {
    child: Option<CommandChild>,
}

#[tauri::command]
async fn start_capture_service(
    app: AppHandle,
    state: State<'_, Mutex<CaptureProcess>>,
) -> Result<(), String> {
    let mut guard = state.lock().unwrap();

    if guard.child.is_some() {
        return Err("capture service already running".into());
    }

    let parent_pid = std::process::id();

    let mut command = app
        .shell()
        .sidecar("clip-service")
        .map_err(|_| "failed to create capture service sidecar")?;

    command = command.arg("--parent-pid").arg(parent_pid.to_string());

    let (mut rx, child) = command.spawn().map_err(|e| e.to_string())?;

    let window = app
        .get_webview_window("main")
        .ok_or("main window not found")?;

    tauri::async_runtime::spawn(async move {
        while let Some(event) = rx.recv().await {
            match event {
                CommandEvent::Stdout(data) => {
                    if let Ok(text) = String::from_utf8(data) {
                        let _ = window.emit("capture-log", text);
                    }
                }
                CommandEvent::Stderr(data) => {
                    if let Ok(text) = String::from_utf8(data) {
                        let _ = window.emit("capture-log", text);
                    }
                }
                CommandEvent::Terminated(_) => {
                    let _ = window.emit("capture-exit", ());
                    break;
                }
                _ => {}
            }
        }
    });

    guard.child = Some(child);
    Ok(())
}

#[tauri::command]
fn stop_capture_service(state: State<'_, Mutex<CaptureProcess>>) -> Result<(), String> {
    let mut guard = state.lock().unwrap();

    if let Some(child) = guard.child.take() {
        child.kill().map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(Mutex::new(CaptureProcess { child: None }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            start_capture_service,
            stop_capture_service
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
