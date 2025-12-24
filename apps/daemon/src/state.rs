use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;

use crate::{gst_capture::GstCapture, ring_buffer::RingBuffer, settings::UserSettings};

pub struct DaemonState {
    pub settings: UserSettings,
    pub buffering: bool,
    pub buffer_seconds: u32,
    pub shutdown_tx: Option<oneshot::Sender<()>>,
    pub capture: Option<GstCapture>,
    pub ring_buffer: Arc<Mutex<RingBuffer>>,
}

pub type SharedState = Arc<Mutex<DaemonState>>;
