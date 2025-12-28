use crate::capture_devices::{AudioDevice, list_microphone_devices};
use axum::Json;

pub async fn get_microphone_devices() -> Json<Vec<AudioDevice>> {
    Json(list_microphone_devices())
}
