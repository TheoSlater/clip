use crate::ffmpeg::capture::{AudioDevice, devices::list_audio_devices};
use axum::Json;

pub async fn get_audio_devices() -> Json<Vec<AudioDevice>> {
    Json(list_audio_devices())
}
