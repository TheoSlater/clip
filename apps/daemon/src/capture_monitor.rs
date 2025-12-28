use std::time::Duration;
use tokio::time::sleep;

use crate::{logger, runtime::restart_capture, state::SharedState};

pub fn spawn_capture_monitor(state: SharedState) {
    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(2)).await;

            let mut guard = state.lock().unwrap();

            // Shutdown path
            if guard.shutdown_tx.is_none() {
                if let Some(mut capture) = guard.capture.take() {
                    capture.stop();
                }
                break;
            }

            // Restart if missing or dead
            let needs_restart = match guard.capture.as_ref() {
                Some(capture) => !capture.is_running(),
                None => true,
            };

            if needs_restart {
                if let Err(err) = restart_capture(&mut guard) {
                    logger::error("capture", format!("restart failed: {}", err));
                }
            }
        }
    });
}
