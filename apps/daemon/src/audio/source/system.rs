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

    pub fn build(&self, pipeline: &gst::Pipeline, volume_value: f32) -> io::Result<AudioSourceOutput> {
        let src = gst::ElementFactory::make("wasapisrc")
            .build()
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "missing wasapisrc"))?;

        src.set_property("loopback", &true);
        src.set_property("provide-clock", &true);
        src.set_property("low-latency", &false);
        src.set_property("do-timestamp", &true);

        let queue = gst::ElementFactory::make("queue")
            .build()
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "missing queue element"))?;

        queue.set_property("max-size-time", &100_000_000u64);
        queue.set_property_from_str("leaky", "downstream");

        let convert = gst::ElementFactory::make("audioconvert")
            .build()
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "missing audioconvert"))?;

        let volume = gst::ElementFactory::make("volume")
            .build()
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "missing volume element"))?;
        let volume_value = volume_value as f64;
        volume.set_property("volume", &volume_value);

        let capsfilter = gst::ElementFactory::make("capsfilter")
            .build()
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "missing capsfilter"))?;

        let caps = gst::Caps::builder("audio/x-raw")
            .field("rate", 48_000i32)
            .field("channels", 2i32)
            .build();
        capsfilter.set_property("caps", &caps);

        pipeline
            .add_many(&[&src, &convert, &volume, &queue, &capsfilter])
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "failed to add elements"))?;

        gst::Element::link_many(&[&src, &convert, &volume, &queue, &capsfilter])
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "failed to link elements"))?;

        Ok(AudioSourceOutput {
            element: capsfilter,
            volume: Some(volume),
        })
    }
}
