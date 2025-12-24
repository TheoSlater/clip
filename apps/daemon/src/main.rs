mod capture_devices;
mod capture_monitor;
mod encoders;
mod gst_capture;
mod ring_buffer;
mod routes;
mod runtime;
mod settings;
mod state;

use axum::Router;

use state::{DaemonState, SharedState};
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio::{net::TcpListener, sync::oneshot};

use crate::{
    capture_devices::{list_audio_devices, list_video_devices},
    capture_monitor::spawn_capture_monitor,
    encoders::list_video_encoders,
    ring_buffer::RingBuffer,
    routes::build_router,
    runtime::restart_capture,
    settings::{apply_startup_fallbacks, default_settings, load_settings, save_settings},
};

#[tokio::main]
async fn main() {
    // --- shutdown signal ---
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    // --- ring buffer ---
    let ring_buffer = Arc::new(Mutex::new(RingBuffer::new(30_000))); // 30s ring buffer

    // --- shared daemon state ---
    let video_devices = list_video_devices();
    let audio_devices = list_audio_devices();
    let encoders = list_video_encoders().expect("failed to enumerate video encoders");

    let loaded_settings = load_settings().expect("failed to load settings");
    let mut settings = match loaded_settings.as_ref() {
        Some(loaded) => {
            println!("[settings] loaded from disk");
            loaded.clone()
        }
        None => {
            let defaults =
                default_settings(&video_devices, &audio_devices, &encoders)
                    .expect("failed to build default settings");
            println!("[settings] created defaults");
            defaults
        }
    };

    let (validated, changes) =
        apply_startup_fallbacks(settings.clone(), &video_devices, &audio_devices, &encoders);
    if !changes.is_empty() {
        for change in &changes {
            println!("[settings] {}", change);
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
    println!("daemon listening on {}", addr);

    let listener: TcpListener = TcpListener::bind(addr)
        .await
        .expect("failed to bind TCP listener");

    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            shutdown_rx.await.ok();
            println!("shutting down daemon");
        })
        .await
        .expect("server error");
}
