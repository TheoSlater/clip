use crate::ffmpeg::capture::AudioDevice;
use axum::Json;

pub async fn get_audio_devices() -> Json<Vec<AudioDevice>> {
    Json(vec![])
}
