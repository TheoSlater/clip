#[derive(Debug, Clone)]
pub enum VideoCaptureSource {
    Screen,
    Monitor { index: u32 },
}

impl VideoCaptureSource {
    pub fn from_device_id(id: &str) -> Option<Self> {
        // macOS example: avf:video:4
        let parts: Vec<&str> = id.split(':').collect();

        match parts.as_slice() {
            ["avf", "video", index] => index
                .parse::<u32>()
                .ok()
                .map(|i| VideoCaptureSource::Monitor { index: i }),
            ["screen", "primary"] => Some(VideoCaptureSource::Screen),
            _ => None,
        }
    }
}

#[cfg(target_os = "windows")]
mod windows {
    use super::VideoCaptureSource;

    pub fn screen_capture_args(source: &VideoCaptureSource) -> Vec<String> {
        match source {
            VideoCaptureSource::Screen => vec![
                "-f".to_string(),
                "gdigrab".to_string(),
                "-draw_mouse".to_string(),
                "1".to_string(),
                "-i".to_string(),
                "desktop".to_string(),
            ],

            VideoCaptureSource::Monitor { index: _ } => {
                // keep your offset logic for now
                vec![
                    "-f".to_string(),
                    "gdigrab".to_string(),
                    "-draw_mouse".to_string(),
                    "1".to_string(),
                    "-i".to_string(),
                    "desktop".to_string(),
                ]
            }
        }
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use super::VideoCaptureSource;

    /// Build avfoundation input arguments for macOS.
    ///
    /// Notes:
    /// - The index must come directly from avfoundation enumeration.
    /// - No framerate is set here (builder owns that).
    pub fn screen_capture_args(source: &VideoCaptureSource) -> Vec<String> {
        match source {
            // Default screen (screen 0)
            VideoCaptureSource::Screen => vec![
                "-f".to_string(),
                "avfoundation".to_string(),
                "-i".to_string(),
                "0".to_string(),
            ],

            // Explicit device selection
            VideoCaptureSource::Monitor { index } => vec![
                "-f".to_string(),
                "avfoundation".to_string(),
                "-i".to_string(),
                index.to_string(),
            ],
        }
    }
}

pub fn screen_capture_args(source: &VideoCaptureSource) -> Vec<String> {
    #[cfg(target_os = "windows")]
    return windows::screen_capture_args(source);

    #[cfg(target_os = "macos")]
    return macos::screen_capture_args(source);
}
