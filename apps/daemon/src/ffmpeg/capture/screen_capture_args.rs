#[derive(Debug, Clone)]
pub enum CaptureSource {
    Screen,
    Monitor { index: u32 },
}

impl CaptureSource {
    pub fn from_device_id(id: &str) -> Option<Self> {
        // macOS example: avf:video:4
        let parts: Vec<&str> = id.split(':').collect();

        match parts.as_slice() {
            ["avf", "video", index] => index
                .parse::<u32>()
                .ok()
                .map(|i| CaptureSource::Monitor { index: i }),
            ["screen", "primary"] => Some(CaptureSource::Screen),
            _ => None,
        }
    }
}

#[cfg(target_os = "windows")]
mod windows {
    use super::CaptureSource;

    pub fn screen_capture_args(source: &CaptureSource) -> Vec<String> {
        match source {
            CaptureSource::Screen => vec![
                "-f".to_string(),
                "gdigrab".to_string(),
                "-draw_mouse".to_string(),
                "1".to_string(),
                "-i".to_string(),
                "desktop".to_string(),
            ],

            CaptureSource::Monitor { index: _ } => {
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
    use super::CaptureSource;

    /// Build avfoundation input arguments for macOS.
    ///
    /// Notes:
    /// - The index must come directly from avfoundation enumeration.
    /// - No framerate is set here (builder owns that).
    pub fn screen_capture_args(source: &CaptureSource) -> Vec<String> {
        match source {
            // Default screen (screen 0)
            CaptureSource::Screen => vec![
                "-f".to_string(),
                "avfoundation".to_string(),
                "-i".to_string(),
                "0".to_string(),
            ],

            // Explicit device selection
            CaptureSource::Monitor { index } => vec![
                "-f".to_string(),
                "avfoundation".to_string(),
                "-i".to_string(),
                index.to_string(),
            ],
        }
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use super::CaptureSource;

    pub fn screen_capture_args(source: &CaptureSource) -> Vec<String> {
        // linux impl
    }
}

pub fn screen_capture_args(source: &CaptureSource) -> Vec<String> {
    #[cfg(target_os = "windows")]
    return windows::screen_capture_args(source);

    #[cfg(target_os = "macos")]
    return macos::screen_capture_args(source);

    #[cfg(target_os = "linux")]
    return linux::screen_capture_args(source);
}
