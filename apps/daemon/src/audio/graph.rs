use std::io;

use gstreamer as gst;

use crate::{
    audio::{encoder::AudioEncoder, mixer::AudioMixer, source::AudioSource},
    settings::UserSettings,
};

pub struct GraphOutput {
    pub element: gst::Element,
}

pub struct AudioGraph {
    pub output: GraphOutput,
}

impl AudioGraph {
    pub fn build(pipeline: &gst::Pipeline, config: &UserSettings) -> io::Result<Option<Self>> {
        let sources = AudioSource::from_settings(config)?;

        if sources.is_empty() {
            return Ok(None);
        }

        let mut built_sources = Vec::new();
        for source in sources {
            built_sources.push(source.build(pipeline)?);
        }

        let mixed = if built_sources.len() == 1 {
            built_sources.into_iter().next().unwrap()
        } else {
            let mixer = AudioMixer::from_settings(config)?.expect("mixer required but not created");

            mixer.build(pipeline, built_sources)?
        };

        let encoder = AudioEncoder::from_settings(config)?;
        let encoded = encoder.build(pipeline, mixed)?;

        Ok(Some(Self { output: encoded }))
    }
}
