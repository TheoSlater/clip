use axum::{Json, extract::State};
use serde::Serialize;

use crate::state::SharedState;

#[derive(Serialize)]
pub struct ShutdownResponse {
    pub shutting_down: bool,
}

pub async fn shutdown(State(state): State<SharedState>) -> Json<ShutdownResponse> {
    let mut guard = state.lock().unwrap();

    if let Some(mut capture) = guard.capture.take() {
        capture.stop();
    }

    if let Some(tx) = guard.shutdown_tx.take() {
        let _ = tx.send(());
    }

    Json(ShutdownResponse {
        shutting_down: true,
    })
}
