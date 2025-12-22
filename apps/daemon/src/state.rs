use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;

use crate::{ffmpeg::process::FFmpegProcess, ring_buffer::RingBuffer};

pub struct DaemonState {
    pub capture_config: CaptureConfig,
    pub buffering: bool,
    pub buffer_seconds: u32,
    pub shutdown_tx: Option<oneshot::Sender<()>>,
    pub ffmpeg: Option<FFmpegProcess>,
    pub ring_buffer: Arc<Mutex<RingBuffer>>,
}

pub type SharedState = Arc<Mutex<DaemonState>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureConfig {
    pub video_device_id: String,
    pub audio_device_id: Option<String>,
    pub framerate: u32,
}
