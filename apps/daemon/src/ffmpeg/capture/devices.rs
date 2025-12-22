use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub enum VideoDeviceKind {
    Screen,
    Camera,
}

#[derive(Debug, Clone, Serialize)]
pub struct VideoDevice {
    pub id: String,
    pub label: String,
    pub kind: VideoDeviceKind,
}

#[derive(Debug, Clone, Serialize)]
pub struct AudioDevice {
    pub id: String,
    pub label: String,
    pub is_input: bool, // mic vs system/loopback
}

#[cfg(target_os = "windows")]
mod windows {
    pub fn list_video_devices() -> Vec<VideoDevice> {
        vec![VideoDevice {
            id: "screen:primary".into(),
            label: "Primary Display".into(),
            kind: VideoDeviceKind::Screen,
        }]
    }

    #[cfg(target_os = "windows")]
    pub fn list_audio_devices() -> Vec<AudioDevice> {
        vec![AudioDevice {
            id: "audio:primary".into(),
            label: "Primary Audio".into(),
            is_input: false,
        }]
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use std::process::Command;

    use crate::ffmpeg::capture::{AudioDevice, VideoDevice, VideoDeviceKind};

    pub fn list_video_devices() -> Vec<VideoDevice> {
        let output = Command::new("ffmpeg")
            .args(["-f", "avfoundation", "-list_devices", "true", "-i", ""])
            .output()
            .expect("failed to run ffmpeg");

        let stderr = String::from_utf8_lossy(&output.stderr);

        let mut devices = Vec::new();
        let mut in_video_section = false;

        for line in stderr.lines() {
            let line = line.trim();

            if line.contains("AVFoundation video devices") {
                in_video_section = true;
                continue;
            }

            if line.contains("AVFoundation audio devices") {
                in_video_section = false;
                continue;
            }

            if !in_video_section {
                continue;
            }

            // We only care about lines that contain TWO bracket groups
            // Example:
            // [AVFoundation input device @ 0x123] [4] Capture screen 0
            let parts: Vec<&str> = line.split(']').collect();
            if parts.len() < 3 {
                continue;
            }

            // parts[1] starts with " [4"
            let index_part = parts[1].trim();
            let index = index_part.trim_start_matches('[').trim();

            let label = parts[2].trim();

            if index.is_empty() || label.is_empty() {
                continue;
            }

            let kind = if label.to_lowercase().contains("screen") {
                VideoDeviceKind::Screen
            } else {
                VideoDeviceKind::Camera
            };

            devices.push(VideoDevice {
                id: format!("avf:video:{}", index),
                label: label.to_string(),
                kind,
            });
        }

        devices
    }

    pub fn list_audio_devices() -> Vec<AudioDevice> {
        let output = Command::new("ffmpeg")
            .args(["-f", "avfoundation", "-list_devices", "true", "-i", ""])
            .output()
            .expect("failed to run ffmpeg");

        let stderr = String::from_utf8_lossy(&output.stderr);

        let mut devices = Vec::new();
        let mut in_audio_section = false;

        for line in stderr.lines() {
            let line = line.trim();

            if line.contains("AVFoundation audio devices") {
                in_audio_section = true;
                continue;
            }

            if line.contains("AVFoundation video devices") {
                in_audio_section = false;
                continue;
            }

            if !in_audio_section {
                continue;
            }

            let parts: Vec<&str> = line.split(']').collect();
            if parts.len() < 3 {
                continue;
            }

            let index_part = parts[1].trim();
            let index = index_part.trim_start_matches('[').trim();
            let label = parts[2].trim();

            if index.is_empty() || label.is_empty() {
                continue;
            }

            devices.push(AudioDevice {
                id: format!("avf:audio:{}", index),
                label: label.to_string(),
                is_input: true, // avfoundation audio devices are inputs
            });
        }

        devices
    }
}

pub fn list_video_devices() -> Vec<VideoDevice> {
    #[cfg(target_os = "windows")]
    return windows::list_video_devices();

    #[cfg(target_os = "macos")]
    return macos::list_video_devices();
}

pub fn list_audio_devices() -> Vec<AudioDevice> {
    #[cfg(target_os = "windows")]
    return windows::list_audio_devices();

    #[cfg(target_os = "macos")]
    return macos::list_audio_devices();
}
