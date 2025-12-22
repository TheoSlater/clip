use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;

use crate::{ffmpeg::process::FFmpegProcess, ring_buffer::RingBuffer};

pub struct DaemonState {
    pub buffering: bool,
    pub buffer_seconds: u32,
    pub clip_count: u32,
    pub shutdown_tx: Option<oneshot::Sender<()>>,
    pub ffmpeg: Option<FFmpegProcess>,
    pub ring_buffer: Arc<Mutex<RingBuffer>>,
}

pub type SharedState = Arc<Mutex<DaemonState>>;
