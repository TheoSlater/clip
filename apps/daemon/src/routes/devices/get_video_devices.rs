use crate::ffmpeg::capture::{VideoDevice, list_video_devices};
use axum::Json;

pub async fn get_video_devices() -> Json<Vec<VideoDevice>> {
    Json(list_video_devices())
}
