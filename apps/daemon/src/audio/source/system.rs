use std::io;

use gst::prelude::*;
use gstreamer as gst;

use crate::settings::UserSettings;

use super::AudioSourceOutput;

pub struct SystemAudioSource;

impl SystemAudioSource {
    pub fn from_settings(_config: &UserSettings) -> io::Result<Self> {
        Ok(Self)
    }

    pub fn build(&self, pipeline: &gst::Pipeline) -> io::Result<AudioSourceOutput> {
        let src = gst::ElementFactory::make("wasapisrc")
            .build()
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "missing wasapisrc"))?;

        src.set_property("loopback", &true);
        src.set_property("provide-clock", &false);
        src.set_property("low-latency", &false);

        let convert = gst::ElementFactory::make("audioconvert")
            .build()
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "missing audioconvert"))?;
        let resample = gst::ElementFactory::make("audioresample")
            .build()
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "missing audioresample"))?;
        let capsfilter = gst::ElementFactory::make("capsfilter")
            .build()
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "missing capsfilter"))?;

        let caps = gst::Caps::builder("audio/x-raw")
            .field("rate", 48_000i32)
            .field("channels", 2i32)
            .build();
        capsfilter.set_property("caps", &caps);

        pipeline
            .add_many(&[&src, &convert, &resample, &capsfilter])
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "failed to add elements"))?;

        gst::Element::link_many(&[&src, &convert, &resample, &capsfilter])
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "failed to link elements"))?;

        Ok(AudioSourceOutput {
            element: capsfilter,
        })
    }
}
