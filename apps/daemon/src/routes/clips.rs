use axum::Json;
use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Serialize)]
pub struct ClipInfo {
    pub filename: String,
    pub size_bytes: u64,
}

pub async fn list_clips() -> Json<Vec<ClipInfo>> {
    let mut clips = Vec::new();
    let clips_dir = Path::new("clips");

    if let Ok(entries) = fs::read_dir(clips_dir) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    if let Some(name) = entry.file_name().to_str() {
                        clips.push(ClipInfo {
                            filename: name.to_string(),
                            size_bytes: metadata.len(),
                        });
                    }
                }
            }
        }
    }

    Json(clips)
}
