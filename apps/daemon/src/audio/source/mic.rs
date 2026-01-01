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

    pub fn build(
        &self,
        pipeline: &gst::Pipeline,
        volume_value: f32,
    ) -> io::Result<AudioSourceOutput> {
        let src = gst::ElementFactory::make("wasapisrc")
            .build()
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "missing wasapisrc"))?;

        src.set_property("do-timestamp", &true);
        src.set_property_from_str("device", &self.device_id);

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

        pipeline
            .add_many(&[&src, &convert, &volume, &queue])
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "failed to add elements"))?;
        gst::Element::link_many(&[&src, &convert, &volume, &queue])
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "failed to link elements"))?;

        Ok(AudioSourceOutput {
            element: queue,
            volume: Some(volume),
        })
    }
}
