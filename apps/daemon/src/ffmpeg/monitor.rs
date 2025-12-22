use std::time::Duration;
use tokio::time::sleep;

use super::process::FFmpegProcess;
use crate::state::SharedState;

pub fn spawn_ffmpeg_monitor(state: SharedState) {
    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(2)).await;

            let mut guard = state.lock().unwrap();

            // Shutdown path
            if guard.shutdown_tx.is_none() {
                if let Some(ffmpeg) = guard.ffmpeg.take() {
                    ffmpeg.kill();
                }
                break;
            }

            // Restart if missing or dead
            let needs_restart = match guard.ffmpeg.as_mut() {
                Some(proc) => !proc.is_running(),
                None => true,
            };

            if needs_restart {
                match FFmpegProcess::spawn() {
                    Ok(mut proc) => {
                        proc.drain_stderr();

                        // start stdout reader when ffmpeg is started
                        let rb = guard.ring_buffer.clone();
                        proc.start_stdout_reader(rb);
                        guard.ffmpeg = Some(proc);

                        println!("[ffmpeg] restarted");
                    }
                    Err(err) => {
                        println!("[ffmpeg] failed to restart: {}", err);
                        guard.ffmpeg = None;
                    }
                }
            }
        }
    });
}
