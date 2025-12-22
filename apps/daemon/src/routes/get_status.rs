use axum::{Json, extract::State};

use crate::state::{CaptureConfig, SharedState};
use serde::Serialize;

#[derive(Serialize)]
pub struct StatusResponse {
    pub capture_config: CaptureConfig,
    pub buffering: bool,
    pub buffer_seconds: u32,
    pub ring_buffer_packets: usize,
}

pub async fn get_status(State(state): State<SharedState>) -> Json<StatusResponse> {
    let guard = state.lock().unwrap();

    Json(StatusResponse {
        capture_config: guard.capture_config.clone(),
        buffering: guard.buffering,
        buffer_seconds: guard.buffer_seconds,
        ring_buffer_packets: guard.ring_buffer.lock().unwrap().len(),
    })
}
