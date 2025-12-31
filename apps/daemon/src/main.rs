mod audio;
mod capture_devices;
mod capture_monitor;
mod encoders;
mod gst_capture;
mod gst_utils;
mod logger;
mod ring_buffer;
mod routes;
mod runtime;
mod settings;
mod state;
mod video;

use axum::Router;

use state::{DaemonState, SharedState};
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio::{net::TcpListener, sync::oneshot};

use crate::{
    capture_devices::{list_microphone_devices, list_video_devices},
    capture_monitor::spawn_capture_monitor,
    encoders::list_video_encoders,
    logger::init_logging,
    ring_buffer::RingBuffer,
    routes::build_router,
    runtime::restart_capture,
    settings::{apply_startup_fallbacks, default_settings, load_settings, save_settings},
};

use std::env;

fn get_parent_pid() -> Option<u32> {
    let mut args = env::args().skip(1);

    while let Some(arg) = args.next() {
        if arg == "--parent-pid" {
            return args.next().and_then(|v| v.parse::<u32>().ok());
        }
    }

    None
}

#[tokio::main]
async fn main() {
    init_logging();

    if let Some(parent_pid) = get_parent_pid() {
        start_parent_watchdog(parent_pid);
    } else {
        logger::warn("system", "no parent pid supplied, watchdog disabled")
    }

    // --- shutdown signal ---
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    // --- ring buffer ---
    let ring_buffer = Arc::new(Mutex::new(RingBuffer::new(30_000))); // 30s ring buffer

    // --- shared daemon state ---
    let video_devices = list_video_devices();
    let microphones = list_microphone_devices();
    let encoders = list_video_encoders().expect("failed to enumerate video encoders");

    let loaded_settings = load_settings().expect("failed to load settings");
    let mut settings = match loaded_settings.as_ref() {
        Some(loaded) => {
            logger::info("settings", "loaded from disk");
            loaded.clone()
        }
        None => {
            let defaults = default_settings(&video_devices, &encoders)
                .expect("failed to build default settings");
            logger::info("settings", "created defaults");
            defaults
        }
    };

    let (validated, changes) =
        apply_startup_fallbacks(settings.clone(), &video_devices, &microphones, &encoders);
    if !changes.is_empty() {
        for change in &changes {
            logger::info("settings", format!("{}", change));
        }
        settings = validated;
        save_settings(&settings).expect("failed to save settings");
    } else if loaded_settings.is_none() {
        save_settings(&settings).expect("failed to save settings");
    }

    let state: SharedState = Arc::new(Mutex::new(DaemonState {
        settings,
        buffering: true,
        buffer_seconds: 0,
        shutdown_tx: Some(shutdown_tx),
        capture: None,
        ring_buffer: ring_buffer.clone(),
    }));

    // --- start capture immediately ---
    {
        let mut guard = state.lock().unwrap();
        restart_capture(&mut guard).expect("failed to start capture")
    }

    // --- spawn background tasks ---
    spawn_capture_monitor(state.clone());

    // --- server ---
    let app: Router = build_router(state);

    let addr: SocketAddr = SocketAddr::from(([127, 0, 0, 1], 43123));
    logger::info("system", format!("daemon listening on {}", addr));

    let listener: TcpListener = TcpListener::bind(addr)
        .await
        .expect("failed to bind TCP listener");

    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            shutdown_rx.await.ok();
            logger::info("system", "shutting down daemon");
        })
        .await
        .expect("server error");
}

fn start_parent_watchdog(parent_pid: u32) {
    std::thread::spawn(move || {
        logger::info(
            "system",
            format!("starting parent watchdog (pid {})", parent_pid),
        );

        loop {
            // SAFETY: OpenProcess returns an invalid handle if the PID is gone
            let alive = unsafe {
                use windows::Win32::Foundation::CloseHandle;
                use windows::Win32::System::Threading::{
                    OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
                };

                match OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, parent_pid) {
                    Ok(handle) => {
                        if let Err(e) = CloseHandle(handle) {
                            logger::debug("watchdog", format!("CloseHandle failed: {:?}", e))
                        }
                        true
                    }
                    Err(_) => false,
                }
            };

            if !alive {
                logger::warn("watchdog", "parent process exited, shutting down daemon");
                std::process::exit(0);
            }

            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    });
}
