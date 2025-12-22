use axum::{
    Router,
    routing::{get, post},
};
use tower_http::cors::{Any, CorsLayer};

use crate::{
    routes::{
        config::set_capture_config,
        devices::{get_audio_devices, get_video_devices},
    },
    state::SharedState,
};

pub mod clip;
pub mod clips;
pub mod config;
pub mod devices;
pub mod get_status;
pub mod shutdown;

pub fn build_router(state: SharedState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/status", get(get_status::get_status))
        .route("/devices/video", get(get_video_devices))
        .route("/devices/audio", get(get_audio_devices))
        .route("/config/capture", post(set_capture_config))
        .route("/clip", post(clip::clip))
        .route("/clips", get(clips::list_clips))
        .route("/shutdown", post(shutdown::shutdown))
        .with_state(state)
        .layer(cors)
}
