use crate::capture_devices::{VideoDevice, list_video_devices};
use axum::Json;

pub async fn get_video_devices() -> Json<Vec<VideoDevice>> {
    let devices = list_video_devices();

    Json(devices)
}
