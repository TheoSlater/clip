use axum::{Json, extract::State, http::StatusCode};
use serde::Serialize;

use crate::{
    capture_devices::{list_audio_devices, list_video_devices},
    encoders::list_video_encoders,
    runtime::restart_capture,
    settings::{UserSettings, save_settings, validate_settings},
    state::SharedState,
};

#[derive(Debug, Serialize)]
pub(crate) struct ErrorResponse {
    message: String,
}

pub async fn update_settings(
    State(state): State<SharedState>,
    Json(new_settings): Json<UserSettings>,
) -> Result<Json<UserSettings>, (StatusCode, Json<ErrorResponse>)> {
    let video_devices = list_video_devices();
    let audio_devices = list_audio_devices();
    let encoders = list_video_encoders().map_err(|err| {
        eprintln!("[settings] failed to list encoders: {}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                message: "failed to list encoders".to_string(),
            }),
        )
    })?;

    if let Err(message) = validate_settings(&new_settings, &video_devices, &audio_devices, &encoders)
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { message }),
        ));
    }

    let mut guard = state.lock().unwrap();
    guard.settings = new_settings.clone();

    save_settings(&guard.settings).map_err(|err| {
        eprintln!("[settings] failed to save: {}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                message: "failed to save settings".to_string(),
            }),
        )
    })?;

    restart_capture(&mut guard).map_err(|err| {
        eprintln!("[settings] restart failed: {}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                message: "failed to restart capture".to_string(),
            }),
        )
    })?;

    println!("[settings] updated and capture restarted");

    Ok(Json(guard.settings.clone()))
}
