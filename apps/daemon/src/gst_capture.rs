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
use gstreamer::glib;
use gstreamer_app as gst_app;

use crate::{
    encoders,
    ring_buffer::{Packet, RingBuffer},
    settings::UserSettings,
};

pub struct GstCapture {
    pipeline: gst::Pipeline,
    has_error: Arc<AtomicBool>,
}

impl GstCapture {
    pub fn start(config: &UserSettings, ring_buffer: Arc<Mutex<RingBuffer>>) -> io::Result<Self> {
        gst::init().map_err(|err| io::Error::new(io::ErrorKind::Other, err.to_string()))?;

        if config.audio_device_id != "loopback" {
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

        let video_capsfilter = make_element("capsfilter")?;

        let encoder_info =
            encoders::find_video_encoder(&config.video_encoder_id)?.ok_or_else(|| {
                io::Error::new(io::ErrorKind::Other, "selected encoder not available")
            })?;
        let requires_d3d11 = encoder_info.required_memory.as_deref() == Some("D3D11Memory");

        if encoder_info.required_memory.as_deref() == Some("D3D12Memory") {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "selected encoder requires D3D12 memory, which is not supported",
            ));
        }

        let caps = if requires_d3d11 {
            let structure = gst::Structure::builder("video/x-raw")
                .field("format", "NV12")
                .field("framerate", gst::Fraction::new(config.framerate as i32, 1))
                .build();
            let features = gst::CapsFeatures::new(["memory:D3D11Memory"]);
            gst::Caps::builder_full_with_features(features.clone())
                .structure_with_features(structure, features)
                .build()
        } else {
            gst::Caps::builder("video/x-raw")
                .field("format", "NV12")
                .field("framerate", gst::Fraction::new(config.framerate as i32, 1))
                .build()
        };

        video_capsfilter.set_property("caps", &caps);

        let video_encoder = make_video_encoder(
            &config.video_encoder_id,
            config.framerate,
            config.bitrate_kbps,
        )?;

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
        appsink.set_property("drop", &true);
        appsink.set_property("max-buffers", &4u32);

        let d3d11download = if requires_d3d11 {
            None
        } else {
            Some(make_element("d3d11download")?)
        };

        let videoconvert = if requires_d3d11 {
            None
        } else {
            Some(make_element("videoconvert")?)
        };

        let mut elements: Vec<&gst::Element> = vec![
            &video_src,
            &d3d11convert,
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
        ];

        if let Some(download) = d3d11download.as_ref() {
            elements.insert(2, download);
        }

        if let Some(convert) = videoconvert.as_ref() {
            let insert_index = if d3d11download.is_some() { 3 } else { 2 };
            elements.insert(insert_index, convert);
        }

        pipeline
            .add_many(&elements)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err.to_string()))?;

        fn link(a: &gst::Element, b: &gst::Element) -> io::Result<()> {
            a.link(b).map_err(|_| {
                io::Error::new(
                    io::ErrorKind::Other,
                    format!("failed to link {} -> {}", a.name(), b.name()),
                )
            })
        }

        link(&video_src, &d3d11convert)?;
        if let Some(download) = d3d11download.as_ref() {
            link(&d3d11convert, download)?;
            if let Some(convert) = videoconvert.as_ref() {
                link(download, convert)?;
                link(convert, &video_capsfilter)?;
            } else {
                link(download, &video_capsfilter)?;
            }
        } else {
            link(&d3d11convert, &video_capsfilter)?;
        }
        link(&video_capsfilter, &video_encoder)?;
        link(&video_encoder, &h264parse)?;
        link(&h264parse, &h264_capsfilter)?;
        link(&h264_capsfilter, &video_queue)?;

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

        let keyframe_ring_buffer = ring_buffer.clone();
        let h264parse_src = h264parse
            .static_pad("src")
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "missing h264parse src pad"))?;
        h264parse_src.add_probe(gst::PadProbeType::BUFFER, move |_, info| {
            if let Some(buffer) = info.buffer() {
                if !buffer.flags().contains(gst::BufferFlags::DELTA_UNIT) {
                    if let Some(pts) = buffer.dts_or_pts() {
                        let mut guard = keyframe_ring_buffer.lock().unwrap();
                        guard.push_keyframe_pts(pts.mseconds());
                    }
                }
            }
            gst::PadProbeReturn::Ok
        });

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

                    let pts_ms = buffer
                        .dts_or_pts()
                        .map(|pts| pts.mseconds())
                        .unwrap_or_else(|| sink_started_at.elapsed().as_millis() as u64);

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

fn make_video_encoder(
    encoder_id: &str,
    framerate: u32,
    bitrate_kbps: u32,
) -> io::Result<gst::Element> {
    let enc = gst::ElementFactory::make(encoder_id).build().map_err(|_| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("missing encoder {}", encoder_id),
        )
    })?;

    let gop_size: u32 = framerate.max(1);

    set_u32_property(&enc, "bitrate", bitrate_kbps);
    set_u32_property(&enc, "gop-size", gop_size);
    set_u32_property(&enc, "key-int-max", gop_size);
    set_bool_property(&enc, "zero-latency", true);
    set_bool_property(&enc, "insert-sps-pps", true);

    Ok(enc)
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
    let Some(spec) = element.find_property(name) else {
        return;
    };

    let ty = spec.value_type();

    if ty == glib::Type::U32 {
        element.set_property(name, &value);
    } else if ty == glib::Type::I32 {
        let v = value.min(i32::MAX as u32) as i32;
        element.set_property(name, &v);
    } else if ty == glib::Type::U64 {
        element.set_property(name, &(value as u64));
    } else if ty == glib::Type::I64 {
        element.set_property(name, &(value as i64));
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
