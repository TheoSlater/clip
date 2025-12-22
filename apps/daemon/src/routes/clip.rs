use axum::{Json, extract::State, http::StatusCode};
use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

use crate::{ring_buffer::Packet, state::SharedState};

#[derive(serde::Serialize)]
pub struct ClipResponse {
    pub filename: String,
    pub packets: usize,
    pub duration_ms: u64,
    pub bytes: usize,
}

pub async fn clip(State(state): State<SharedState>) -> Result<Json<ClipResponse>, StatusCode> {
    // Snapshot the entire ring buffer
    let ring_buffer = {
        let guard = state.lock().unwrap();
        guard.ring_buffer.clone()
    };

    let packets: Vec<Packet> = ring_buffer.lock().unwrap().snapshot();

    if packets.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Generate filename
    let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
    let filename = format!("clip-{}.ts", timestamp);

    let mut path = PathBuf::from("clips");
    fs::create_dir_all(&path).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    path.push(&filename);

    // Write packets to disk
    let mut file = File::create(&path).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut bytes_written = 0usize;

    for packet in &packets {
        file.write_all(&packet.data)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        bytes_written += packet.data.len();
    }

    // Update the clip count
    {
        let mut guard = state.lock().unwrap();
        guard.clip_count += 1;
    }

    // Compute duration
    let duration_ms = {
        let guard = state.lock().unwrap();
        guard.ring_buffer.lock().unwrap().duration_ms()
    };

    Ok(Json(ClipResponse {
        filename,
        packets: packets.len(),
        duration_ms,
        bytes: bytes_written,
    }))
}
