use axum::{
    Router,
    routing::{get, post},
};
use tower_http::cors::{Any, CorsLayer};

use crate::{
    routes::{
        devices::{get_audio_devices, get_video_devices},
        encoders::get_video_encoders,
        settings::{get_settings, update_settings},
    },
    state::SharedState,
};

pub mod clip;
pub mod clips;
pub mod devices;
pub mod encoders;
pub mod get_status;
pub mod settings;
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
        .route("/encoders/video", get(get_video_encoders))
        .route("/settings", get(get_settings))
        .route("/settings", post(update_settings))
        .route("/clip", post(clip::clip))
        .route("/clips", get(clips::list_clips))
        .route("/shutdown", post(shutdown::shutdown))
        .with_state(state)
        .layer(cors)
}
