use std::{
    io,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Instant,
};

use gst::prelude::*;
use gstreamer as gst;
use gstreamer_app as gst_app;

use crate::{
    ring_buffer::{Packet, RingBuffer},
    state::CaptureConfig,
};

pub struct GstCapture {
    pipeline: gst::Pipeline,
    has_error: Arc<AtomicBool>,
}

impl GstCapture {
    pub fn start(config: &CaptureConfig, ring_buffer: Arc<Mutex<RingBuffer>>) -> io::Result<Self> {
        gst::init().map_err(|err| io::Error::new(io::ErrorKind::Other, err.to_string()))?;

        if config.audio_device_id.as_deref() != Some("loopback") {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "audio device must be loopback",
            ));
        }

        let pipeline = gst::Pipeline::new();
        let has_error = Arc::new(AtomicBool::new(false));

        let video_src = make_element("d3d11screencapturesrc")?;
        set_bool_property(&video_src, "do-timestamp", true);
        if let Some(monitor) = monitor_index_from_id(&config.video_device_id) {
            set_i32_property(&video_src, "monitor-index", monitor);
        }

        let d3d11convert = make_element("d3d11convert")?;
        let d3d11download = make_element("d3d11download")?;
        let videoconvert = make_element("videoconvert")?;
        let videorate = make_element("videorate")?;
        let video_capsfilter = make_element("capsfilter")?;
        let video_caps = gst::Caps::builder("video/x-raw")
            .field("format", "I420")
            .field("framerate", gst::Fraction::new(config.framerate as i32, 1))
            .build();
        video_capsfilter.set_property("caps", &video_caps);

        let video_encoder = make_video_encoder(config.framerate)?;
        let h264parse = make_element("h264parse")?;
        set_i32_property(&h264parse, "config-interval", 1);
        let h264_capsfilter = make_element("capsfilter")?;
        let h264_caps = gst::Caps::builder("video/x-h264")
            .field("stream-format", "byte-stream")
            .field("alignment", "au")
            .build();
        h264_capsfilter.set_property("caps", &h264_caps);
        let video_queue = make_queue()?;

        let audio_src = make_element("wasapisrc")?;
        set_bool_property(&audio_src, "loopback", true);
        set_bool_property(&audio_src, "do-timestamp", true);
        set_bool_property(&audio_src, "provide-clock", true);
        set_bool_property(&audio_src, "low-latency", false);

        let audioconvert = make_element("audioconvert")?;
        let audioresample = make_element("audioresample")?;
        let audiorate = make_element("audiorate")?;
        let audio_capsfilter = make_element("capsfilter")?;
        let audio_caps = gst::Caps::builder("audio/x-raw")
            .field("rate", 48_000i32)
            .field("channels", 2i32)
            .build();
        audio_capsfilter.set_property("caps", &audio_caps);

        let audio_encoder = make_audio_encoder()?;
        let aacparse = make_element("aacparse")?;
        set_str_property(&aacparse, "output-format", "adts");
        set_str_property(&aacparse, "format", "adts");
        let audio_queue = make_queue()?;

        let mux = make_element("mpegtsmux")?;
        set_bool_property(&mux, "streamable", true);

        let appsink = make_element("appsink")?;
        let appsink = appsink
            .downcast::<gst_app::AppSink>()
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "appsink downcast failed"))?;

        appsink.set_property("emit-signals", &true);
        appsink.set_property("sync", &false);
        appsink.set_property("async", &false);
        appsink.set_property("drop", &false);
        appsink.set_property("max-buffers", &0u32);

        pipeline
            .add_many(&[
                &video_src,
                &d3d11convert,
                &d3d11download,
                &videoconvert,
                &videorate,
                &video_capsfilter,
                &video_encoder,
                &h264parse,
                &h264_capsfilter,
                &video_queue,
                &audio_src,
                &audioconvert,
                &audioresample,
                &audiorate,
                &audio_capsfilter,
                &audio_encoder,
                &aacparse,
                &audio_queue,
                &mux,
                appsink.upcast_ref(),
            ])
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err.to_string()))?;

        gst::Element::link_many(&[
            &video_src,
            &d3d11convert,
            &d3d11download,
            &videoconvert,
            &videorate,
            &video_capsfilter,
            &video_encoder,
            &h264parse,
            &h264_capsfilter,
            &video_queue,
        ])
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "failed to link video chain"))?;

        gst::Element::link_many(&[
            &audio_src,
            &audioconvert,
            &audioresample,
            &audiorate,
            &audio_capsfilter,
            &audio_encoder,
            &aacparse,
            &audio_queue,
        ])
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "failed to link audio chain"))?;

        link_queue_to_mux(&video_queue, &mux, "video")?;
        link_queue_to_mux(&audio_queue, &mux, "audio")?;

        mux.link(&appsink)
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "failed to link mux to appsink"))?;

        let started_at = Instant::now();
        let ring_buffer_clone = ring_buffer.clone();
        let sink_started_at = started_at;

        appsink.set_callbacks(
            gst_app::AppSinkCallbacks::builder()
                .new_sample(move |sink| {
                    let sample = match sink.pull_sample() {
                        Ok(sample) => sample,
                        Err(_) => return Err(gst::FlowError::Error),
                    };

                    let buffer = match sample.buffer() {
                        Some(buffer) => buffer,
                        None => return Ok(gst::FlowSuccess::Ok),
                    };

                    let pts_ms = sink_started_at.elapsed().as_millis() as u64;

                    let map = buffer.map_readable().map_err(|_| gst::FlowError::Error)?;
                    let data = map.as_slice().to_vec();

                    if let Ok(mut rb) = ring_buffer_clone.lock() {
                        rb.push(Packet { pts_ms, data });
                    }

                    Ok(gst::FlowSuccess::Ok)
                })
                .build(),
        );

        attach_bus_logger(&pipeline, has_error.clone())?;

        let state_change = pipeline.set_state(gst::State::Playing);
        if state_change.is_err() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "failed to start GStreamer pipeline",
            ));
        }

        Ok(Self {
            pipeline,
            has_error,
        })
    }

    pub fn is_running(&self) -> bool {
        if self.has_error.load(Ordering::SeqCst) {
            return false;
        }

        self.pipeline.current_state() != gst::State::Null
    }

    pub fn stop(self) {
        let _ = self.pipeline.set_state(gst::State::Null);
    }
}

fn make_element(name: &str) -> io::Result<gst::Element> {
    gst::ElementFactory::make(name)
        .build()
        .map_err(|_| io::Error::new(io::ErrorKind::Other, format!("missing element {}", name)))
}

fn make_video_encoder(framerate: u32) -> io::Result<gst::Element> {
    if let Ok(enc) = gst::ElementFactory::make("x264enc").build() {
        let bitrate = 20_000u32;
        set_str_property(&enc, "bitrate", &bitrate.to_string());
        set_str_property(&enc, "speed-preset", "medium");
        set_bool_property(&enc, "byte-stream", true);

        let key_int_max = framerate.saturating_mul(2);
        set_str_property(&enc, "key-int-max", &key_int_max.to_string());

        return Ok(enc);
    }

    if let Ok(enc) = gst::ElementFactory::make("openh264enc").build() {
        return Ok(enc);
    }

    Err(io::Error::new(
        io::ErrorKind::Other,
        "missing H.264 encoder (x264enc or openh264enc)",
    ))
}

fn make_audio_encoder() -> io::Result<gst::Element> {
    if let Ok(enc) = gst::ElementFactory::make("voaacenc").build() {
        set_str_property(&enc, "bitrate", "192000");
        return Ok(enc);
    }

    if let Ok(enc) = gst::ElementFactory::make("avenc_aac").build() {
        set_str_property(&enc, "bitrate", "192000");
        return Ok(enc);
    }

    Err(io::Error::new(
        io::ErrorKind::Other,
        "missing AAC encoder (voaacenc or avenc_aac)",
    ))
}

fn make_queue() -> io::Result<gst::Element> {
    let queue = make_element("queue")?;
    set_u32_property(&queue, "max-size-buffers", 0);
    set_u32_property(&queue, "max-size-bytes", 0);
    set_u64_property(&queue, "max-size-time", 1_000_000_000);
    Ok(queue)
}

fn link_queue_to_mux(queue: &gst::Element, mux: &gst::Element, label: &str) -> io::Result<()> {
    let queue_src = queue
        .static_pad("src")
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "missing queue src pad"))?;
    let mux_sink = mux
        .request_pad_simple("sink_%d")
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "missing mux sink pad"))?;

    queue_src.link(&mux_sink).map_err(|_| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("failed to link {} to mux", label),
        )
    })?;

    Ok(())
}

fn attach_bus_logger(pipeline: &gst::Pipeline, has_error: Arc<AtomicBool>) -> io::Result<()> {
    let bus = pipeline
        .bus()
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "missing pipeline bus"))?;
    let pipeline_name = pipeline.name();

    std::thread::spawn(move || {
        loop {
            let message = bus.timed_pop(gst::ClockTime::NONE);
            let Some(message) = message else {
                continue;
            };

            match message.view() {
                gst::MessageView::Error(err) => {
                    let src = err
                        .src()
                        .map(|s| s.path_string())
                        .unwrap_or_else(|| "unknown".to_string().into());
                    eprintln!("[gst] error from {}: {}", src, err.error());
                    if let Some(debug) = err.debug() {
                        eprintln!("[gst] debug: {}", debug);
                    }
                    has_error.store(true, Ordering::SeqCst);
                    break;
                }
                gst::MessageView::Warning(warn) => {
                    let src = warn
                        .src()
                        .map(|s| s.path_string())
                        .unwrap_or_else(|| "unknown".to_string().into());
                    eprintln!("[gst] warning from {}: {}", src, warn.error());
                    if let Some(debug) = warn.debug() {
                        eprintln!("[gst] debug: {}", debug);
                    }
                }
                gst::MessageView::StateChanged(state) => {
                    if message
                        .src()
                        .map(|s| s.name() == pipeline_name)
                        .unwrap_or(false)
                    {
                        eprintln!(
                            "[gst] pipeline state: {:?} -> {:?}",
                            state.old(),
                            state.current()
                        );
                    }
                }
                gst::MessageView::Eos(..) => {
                    eprintln!("[gst] eos");
                    has_error.store(true, Ordering::SeqCst);
                    break;
                }
                _ => {}
            }
        }
    });

    Ok(())
}

fn monitor_index_from_id(id: &str) -> Option<i32> {
    let mut parts = id.split(':');
    let kind = parts.next()?;
    let index = parts.next()?;
    if kind != "screen" {
        return None;
    }

    index.parse::<i32>().ok()
}

fn set_i32_property(element: &gst::Element, name: &str, value: i32) {
    if element.find_property(name).is_some() {
        element.set_property(name, &value);
    }
}

fn set_u32_property(element: &gst::Element, name: &str, value: u32) {
    if element.find_property(name).is_some() {
        element.set_property(name, &value);
    }
}

fn set_u64_property(element: &gst::Element, name: &str, value: u64) {
    if element.find_property(name).is_some() {
        element.set_property(name, &value);
    }
}

fn set_bool_property(element: &gst::Element, name: &str, value: bool) {
    if element.find_property(name).is_some() {
        element.set_property(name, &value);
    }
}

fn set_str_property(element: &gst::Element, name: &str, value: &str) {
    if element.find_property(name).is_some() {
        element.set_property_from_str(name, value);
    }
}
