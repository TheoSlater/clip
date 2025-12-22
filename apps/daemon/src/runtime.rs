use std::io;

use crate::{
    ffmpeg::{builder::build_ffmpeg_args, process::FFmpegProcess},
    state::DaemonState,
};

/// Start or restart capture to match the current CaptureConfig.
///
/// This function is the ONLY place that:
/// - spawns ffmpeg
/// - kills ffmpeg
/// - wires stdout/stderr
/// - clears the ring buffer
pub fn restart_capture(state: &mut DaemonState) -> Result<(), io::Error> {
    // Kill existing ffmpeg (if any)
    if let Some(old) = state.ffmpeg.take() {
        println!("[ffmpeg] killing existing process");
        old.kill();
    }

    // Clear ring buffer (new timeline)
    {
        let mut rb = state.ring_buffer.lock().unwrap();
        rb.clear();
    }

    // Build ffmpeg args from config
    let args = build_ffmpeg_args(&state.capture_config);

    // Spawn ffmpeg
    let mut proc = FFmpegProcess::spawn(args)?;

    // Attach IO
    proc.drain_stderr();
    proc.start_stdout_reader(state.ring_buffer.clone());

    // Store handle
    state.ffmpeg = Some(proc);

    println!("[ffmpeg] capture started");

    Ok(())
}
