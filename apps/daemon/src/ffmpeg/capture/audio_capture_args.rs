#[derive(Debug, Clone)]

pub enum AudioCaptureSource {
    Device { id: String },
}

impl AudioCaptureSource {
    pub fn from_device_id(id: &str) -> Option<Self> {
        Some(AudioCaptureSource::Device { id: id.to_string() })
    }
}

#[cfg(target_os = "windows")]
mod windows {
    pub fn audio_capture_args(_source: &AudioSource) -> Vec<String> {
        vec![]
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use super::AudioCaptureSource;

    pub fn audio_capture_args(source: &AudioCaptureSource) -> Vec<String> {
        match source {
            AudioCaptureSource::Device { id } => {
                // macOS avfoundation audio ids look like: avf:audio:1
                let index = id.split(":").last().unwrap();

                vec![
                    "-f".to_string(),
                    "avfoundation".to_string(),
                    "-i".to_string(),
                    format!(":{}", index),
                ]
            }
        }
    }
}

pub fn audio_capture_args(source: &AudioCaptureSource) -> Vec<String> {
    #[cfg(target_os = "macos")]
    return macos::audio_capture_args(source);

    #[cfg(target_os = "windows")]
    return windows::audio_capture_args(source);
}
