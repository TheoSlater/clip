use std::io;

use gst::prelude::*;
use gstreamer as gst;

use super::AudioSourceOutput;

pub struct MicAudioSource {
    device_id: String,
}

impl MicAudioSource {
    pub fn from_device(device_id: &str) -> io::Result<Self> {
        if device_id.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "mic device id is empty",
            ));
        }

        Ok(Self {
            device_id: device_id.to_string(),
        })
    }

    pub fn build(&self, pipeline: &gst::Pipeline) -> io::Result<AudioSourceOutput> {
        let src = gst::ElementFactory::make("wasapisrc")
            .build()
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "missing wasapisrc"))?;

        // Mic, not loopback
        src.set_property("loopback", &false);
        src.set_property("provide-clock", &false);
        src.set_property("low-latency", &false);
        src.set_property_from_str("device", &self.device_id);

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
