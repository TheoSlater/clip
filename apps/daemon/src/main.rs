mod ffmpeg;
mod ring_buffer;
mod routes;
mod state;

use axum::Router;

use state::{DaemonState, SharedState};
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{net::TcpListener, sync::oneshot, time::sleep};

use crate::{
    ffmpeg::{monitor::spawn_ffmpeg_monitor, process::FFmpegProcess},
    ring_buffer::RingBuffer,
    routes::build_router,
};

#[tokio::main]
async fn main() {
    // --- shutdown signal ---
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    // --- ring buffer ---
    let ring_buffer = Arc::new(Mutex::new(RingBuffer::new(30_000))); // 30s ring buffer

    // --- shared daemon state ---
    let state: SharedState = Arc::new(Mutex::new(DaemonState {
        buffering: true,
        buffer_seconds: 0,
        clip_count: 0,
        shutdown_tx: Some(shutdown_tx),
        ffmpeg: None,
        ring_buffer: ring_buffer.clone(),
    }));

    // --- start ffmpeg immediately ---
    {
        let mut guard = state.lock().unwrap();
        match FFmpegProcess::spawn() {
            Ok(mut proc) => {
                proc.drain_stderr();

                // start stdout reader when ffmpeg is started
                let rb = guard.ring_buffer.clone();
                proc.start_stdout_reader(rb);

                guard.ffmpeg = Some(proc);

                println!("[ffmpeg] started");
            }
            Err(err) => {
                println!("[ffmpeg] failed to start: {}", err);
                guard.ffmpeg = None;
            }
        }
    }

    // --- spawn background tasks ---
    spawn_buffer_simulator(state.clone());
    spawn_ffmpeg_monitor(state.clone());

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

fn spawn_buffer_simulator(state: SharedState) {
    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(1)).await;

            let mut guard = state.lock().unwrap();

            if guard.shutdown_tx.is_none() {
                break;
            }

            if guard.buffering {
                guard.buffer_seconds += 1;

                // Cap the buffer at 30 seconds (simulate ring buffer)
                if guard.buffer_seconds > 30 {
                    guard.buffer_seconds = 30;
                }
            }
        }
    });
}
