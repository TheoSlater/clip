use axum::{Json, extract::State};

use crate::{logger, settings::UserSettings, state::SharedState};
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct StatusResponse {
    pub settings: UserSettings,
    pub buffering: bool,
    pub buffer_seconds: u32,
    pub ring_buffer_packets: usize,
}

pub async fn get_status(State(state): State<SharedState>) -> Json<StatusResponse> {
    let guard = state.lock().unwrap();

    let status = StatusResponse {
        settings: guard.settings.clone(),
        buffering: guard.buffering,
        buffer_seconds: guard.buffer_seconds,
        ring_buffer_packets: guard.ring_buffer.lock().unwrap().len(),
    };

    logger::info("status", format!("Status response: {:?}", status));

    Json(status)
}
