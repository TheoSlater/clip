use std::io;

use gst::prelude::*;
use gstreamer as gst;

use crate::{
    audio::{encoder::AudioEncoder, mixer::AudioMixer, source::AudioSource},
    settings::UserSettings,
};

pub struct AudioVolumes {
    pub system: Option<gst::Element>,
    pub mic: Option<gst::Element>,
}

pub struct GraphOutput {
    pub element: gst::Element,
}

pub struct AudioGraph {
    pub output: GraphOutput,
    pub volumes: AudioVolumes,
}

impl AudioGraph {
    pub fn build(pipeline: &gst::Pipeline, config: &UserSettings) -> io::Result<Option<Self>> {
        let sources = AudioSource::from_settings(config)?;

        if sources.is_empty() {
            return Ok(None);
        }

        let mut built_sources = Vec::new();
        let mut volumes = AudioVolumes {
            system: None,
            mic: None,
        };
        for source in sources {
            match source {
                AudioSource::System(s) => {
                    let built = s.build(pipeline, config.system_audio_volume)?;
                    volumes.system = built.volume.clone();
                    built_sources.push(built);
                }
                AudioSource::Mic(s) => {
                    let built = s.build(pipeline, config.mic_volume)?;
                    volumes.mic = built.volume.clone();
                    built_sources.push(built);
                }
            }
        }

        let mixed = if built_sources.len() == 1 {
            built_sources.into_iter().next().unwrap()
        } else {
            let mixer = AudioMixer::from_settings(config)?.expect("mixer required but not created");

            mixer.build(pipeline, built_sources)?
        };

        let resample = gst::ElementFactory::make("audioresample")
            .build()
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "missing audioresample"))?;

        resample.set_property("quality", &10i32);

        let capsfilter = gst::ElementFactory::make("capsfilter")
            .build()
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "missing capsfilter"))?;

        let caps = gst::Caps::builder("audio/x-raw")
            .field("rate", 48_000i32)
            .field("channels", 2i32)
            .build();

        pipeline.add_many(&[&resample, &capsfilter]).map_err(|_| {
            io::Error::new(
                io::ErrorKind::Other,
                "failed to add audio post-mix elements",
            )
        })?;

        gst::Element::link_many(&[&mixed.element, &resample, &capsfilter]).map_err(|_| {
            io::Error::new(io::ErrorKind::Other, "failed to link audio post-mix chain")
        })?;

        capsfilter.set_property("caps", &caps);

        let post_mix = super::source::AudioSourceOutput {
            element: capsfilter,
            volume: None,
        };

        let encoder = AudioEncoder::from_settings(config)?;
        let encoded = encoder.build(pipeline, post_mix)?;

        Ok(Some(Self {
            output: encoded,
            volumes,
        }))
    }
}
