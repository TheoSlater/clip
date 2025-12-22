use std::{
    io::{BufRead, BufReader, Read},
    process::{Child, ChildStderr, ChildStdout, Command, Stdio},
    sync::{Arc, Mutex},
};

use crate::{
    ffmpeg::capture::{CaptureSource, screen_capture_args},
    ring_buffer::{Packet, RingBuffer},
};

pub struct FFmpegProcess {
    child: Child,
    stderr: Option<ChildStderr>,
    stdout: Option<ChildStdout>,
    start_time: std::time::Instant,
}

impl FFmpegProcess {
    pub fn spawn() -> std::io::Result<Self> {
        let capture = CaptureSource::Screen;

        let mut args: Vec<&str> = Vec::new();

        // global
        args.extend(["-hide_banner", "-loglevel", "error", "-re"]);

        // input (platform-specific)
        args.extend(screen_capture_args(&capture));

        // encoding
        args.extend([
            "-fflags",
            "+genpts",
            "-c:v",
            "libx264",
            "-preset",
            "ultrafast",
            "-tune",
            "zerolatency",
        ]);

        // output
        args.extend([
            "-flush_packets",
            "1",
            "-muxdelay",
            "0",
            "-muxpreload",
            "0",
            "-f",
            "mpegts",
            "pipe:1",
        ]);

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
                let mut buf = [0u8; 188 * 7]; // TS packet multiple

                loop {
                    match stdout.read(&mut buf) {
                        Ok(0) => break, // FFmpeg exited
                        Ok(n) => {
                            let pts_ms = start.elapsed().as_millis() as u64;

                            let packet = Packet {
                                pts_ms,
                                data: buf[..n].to_vec(),
                            };

                            if let Ok(mut rb) = ring_buffer.lock() {
                                rb.push(packet);
                            }
                        }
                        Err(_) => break,
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
