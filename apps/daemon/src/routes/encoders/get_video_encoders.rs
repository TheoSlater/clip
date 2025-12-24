use axum::{Json, http::StatusCode};

use crate::encoders::{VideoEncoderDescriptor, list_video_encoders};

pub async fn get_video_encoders() -> Result<Json<Vec<VideoEncoderDescriptor>>, StatusCode> {
    match list_video_encoders() {
        Ok(encoders) => Ok(Json(encoders)),
        Err(err) => {
            eprintln!("[encoders] failed to list encoders: {}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
