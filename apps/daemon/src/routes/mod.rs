use axum::{
    Router,
    routing::{get, post},
};
use tower_http::cors::{Any, CorsLayer};

use crate::state::SharedState;

pub mod clip;
pub mod clips;
pub mod get_status;
pub mod shutdown;

pub fn build_router(state: SharedState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/status", get(get_status::get_status))
        .route("/clip", post(clip::clip))
        .route("/clips", get(clips::list_clips))
        .route("/shutdown", post(shutdown::shutdown))
        .with_state(state)
        .layer(cors)
}
