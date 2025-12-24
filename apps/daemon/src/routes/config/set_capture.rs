use crate::{
    capture_devices::list_video_devices,
    runtime::restart_capture,
    state::{CaptureConfig, SharedState},
};
use axum::{Json, extract::State, http::StatusCode};

pub async fn set_capture_config(
    State(state): State<SharedState>,
    Json(new_config): Json<CaptureConfig>,
) -> Result<StatusCode, StatusCode> {
    let devices = list_video_devices();

    // Validate device exists
    if !devices.iter().any(|d| d.id == new_config.video_device_id) {
        return Err(StatusCode::BAD_REQUEST);
    }

    if let Some(audio_id) = &new_config.audio_device_id {
        if audio_id != "loopback" {
            return Err(StatusCode::BAD_REQUEST);
        }
    }

    let mut guard = state.lock().unwrap();

    // Save config
    guard.capture_config = new_config.clone();

    // Restart capture with new config
    restart_capture(&mut guard).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}
