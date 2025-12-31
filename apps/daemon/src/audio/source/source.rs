use std::io;

use gstreamer as gst;

use crate::settings::UserSettings;

use super::{mic::MicAudioSource, system::SystemAudioSource};

pub enum AudioSource {
    System(SystemAudioSource),
    Mic(MicAudioSource),
}

pub struct AudioSourceOutput {
    pub element: gst::Element,
}

// Audio sources should do the following:
// - Capture
// - Convert format
// - Ensure channel layout
// They should not do any resampling, or make any timing related decisions
impl AudioSource {
    pub fn from_settings(config: &UserSettings) -> io::Result<Vec<Self>> {
        let mut sources = Vec::new();

        if config.system_audio_enabled {
            sources.push(AudioSource::System(SystemAudioSource::from_settings(
                config,
            )?));
        }

        if let Some(id) = config.mic_device_id.as_ref().filter(|s| !s.is_empty()) {
            sources.push(AudioSource::Mic(MicAudioSource::from_device(id)?));
        }

        Ok(sources)
    }

    pub fn build(self, pipeline: &gst::Pipeline) -> io::Result<AudioSourceOutput> {
        match self {
            AudioSource::System(s) => s.build(pipeline),
            AudioSource::Mic(s) => s.build(pipeline),
        }
    }
}
