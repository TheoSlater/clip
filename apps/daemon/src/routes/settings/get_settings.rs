use axum::{Json, extract::State};

use crate::{settings::UserSettings, state::SharedState};

pub async fn get_settings(State(state): State<SharedState>) -> Json<UserSettings> {
    let guard = state.lock().unwrap();
    Json(guard.settings.clone())
}
