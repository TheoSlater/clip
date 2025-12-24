mod capture_devices;
mod capture_monitor;
mod gst_capture;
mod ring_buffer;
mod routes;
mod runtime;
mod state;

use axum::Router;

use state::{DaemonState, SharedState};
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio::{net::TcpListener, sync::oneshot};

use crate::{
    capture_devices::{VideoDeviceKind, list_video_devices},
    capture_monitor::spawn_capture_monitor,
    ring_buffer::RingBuffer,
    routes::build_router,
    runtime::restart_capture,
    state::CaptureConfig,
};

#[tokio::main]
async fn main() {
    // --- shutdown signal ---
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    // --- ring buffer ---
    let ring_buffer = Arc::new(Mutex::new(RingBuffer::new(30_000))); // 30s ring buffer

    // --- shared daemon state ---
    let initial_devices = list_video_devices();
    let default_video = initial_devices
        .iter()
        .find(|d| matches!(d.kind, VideoDeviceKind::Screen))
        .or_else(|| initial_devices.first())
        .expect("no video devices found");

    let state: SharedState = Arc::new(Mutex::new(DaemonState {
        capture_config: CaptureConfig {
            video_device_id: default_video.id.clone(),
            audio_device_id: Some("loopback".to_string()),
            framerate: 60,
        },
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
