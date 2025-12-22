use std::{
    io::{BufRead, BufReader, Read},
    process::{Child, ChildStderr, ChildStdout, Command, Stdio},
    sync::{Arc, Mutex},
};

use crate::ring_buffer::{Packet, RingBuffer};

pub struct FFmpegProcess {
    child: Child,
    stderr: Option<ChildStderr>,
    stdout: Option<ChildStdout>,
    start_time: std::time::Instant,
}

impl FFmpegProcess {
    pub fn spawn(args: Vec<String>) -> std::io::Result<Self> {
        let mut child = Command::new("ffmpeg")
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        Ok(Self {
            child,
            stdout,
            stderr,
            start_time: std::time::Instant::now(),
        })
    }

    pub fn start_stdout_reader(&mut self, ring_buffer: Arc<Mutex<RingBuffer>>) {
        if let Some(mut stdout) = self.stdout.take() {
            let start = self.start_time;

            std::thread::spawn(move || {
                let mut buf = [0u8; 188 * 7];

                loop {
                    match stdout.read(&mut buf) {
                        Ok(0) | Err(_) => break,
                        Ok(n) => {
                            let packet = Packet {
                                pts_ms: start.elapsed().as_millis() as u64,
                                data: buf[..n].to_vec(),
                            };

                            if let Ok(mut rb) = ring_buffer.lock() {
                                rb.push(packet);
                            }
                        }
                    }
                }
            });
        }
    }

    pub fn drain_stderr(&mut self) {
        if let Some(stderr) = self.stderr.take() {
            std::thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines().flatten() {
                    println!("[ffmpeg] {}", line);
                }
            });
        }
    }

    pub fn is_running(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }

    pub fn kill(mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
