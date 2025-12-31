use std::io;

use gst::prelude::*;
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
        };

        let encoder = AudioEncoder::from_settings(config)?;
        let encoded = encoder.build(pipeline, post_mix)?;

        Ok(Some(Self { output: encoded }))
    }
}
