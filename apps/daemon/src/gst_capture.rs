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

use crate::audio::AudioGraph;
use crate::video::VideoGraph;

use crate::{
    logger,
    ring_buffer::{Packet, RingBuffer},
    settings::UserSettings,
};

/// Capture core boundary:
/// - Owns the GStreamer pipeline lifecycle and elements.
/// - Emits encoded packets into the ring buffer.
/// - Exposes explicit capture state (Starting/Running/Failed/Stopped).
/// - Does NOT know about routes, UI state, or settings mutation.
#[derive(Debug, Clone)]
pub enum CaptureState {
    Starting,
    Running,
    Failed(String),
    Stopped,
}

pub struct GstCapture {
    pipeline: gst::Pipeline,
    appsink: gst_app::AppSink,
    state: Arc<Mutex<CaptureState>>,
    stop_flag: Arc<AtomicBool>,
    bus_thread: Option<std::thread::JoinHandle<()>>,
    callback_guard: Arc<AtomicBool>,
}

impl GstCapture {
    pub fn start(config: &UserSettings, ring_buffer: Arc<Mutex<RingBuffer>>) -> io::Result<Self> {
        let callback_guard = Arc::new(AtomicBool::new(false));

        gst::init().map_err(|err| io::Error::new(io::ErrorKind::Other, err.to_string()))?;

        validate_config(config)?;

        let pipeline = gst::Pipeline::new();
        let state = Arc::new(Mutex::new(CaptureState::Starting));
        let stop_flag = Arc::new(AtomicBool::new(false));

        let mux = make_element("mpegtsmux")?;
        set_bool_property(&mux, "streamable", true);

        pipeline
            .add(&mux)
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to add mux"))?;

        let video = VideoGraph::build(&pipeline, config)?;
        let audio = AudioGraph::build(&pipeline, config)?;

        link_queue_to_mux(&video.output.element, &mux, "video")?;

        if let Some(audio) = audio {
            if let Some(src_pad) = audio.output.element.static_pad("src") {
                let caps = src_pad.current_caps();
                logger::info("audio", format!("audio caps before mux: {:?}", caps));
            } else {
                logger::warn("audio", "audio output has no src pad");
            }
            link_queue_to_mux(&audio.output.element, &mux, "audio")?;
        }

        let appsink = make_element("appsink")?;
        let appsink = appsink
            .downcast::<gst_app::AppSink>()
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "appsink downcast failed"))?;

        appsink.set_property("emit-signals", &true);
        appsink.set_property("sync", &false);
        appsink.set_property("async", &false);
        appsink.set_property("drop", &true);
        appsink.set_property("max-buffers", &4u32);

        pipeline
            .add(&appsink)
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to add appsink"))?;

        mux.link(&appsink)
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "failed to link mux to appsink"))?;

        video.attach_keyframe_tracker(ring_buffer.clone())?;

        let started_at = Instant::now();
        let ring_buffer_clone = ring_buffer.clone();

        let callback_guard_clone = callback_guard.clone();

        appsink.set_callbacks(
            gst_app::AppSinkCallbacks::builder()
                .new_sample(move |sink| {
                    if callback_guard_clone.load(Ordering::SeqCst) {
                        return Ok(gst::FlowSuccess::Ok);
                    }

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
                        .unwrap_or_else(|| started_at.elapsed().as_millis() as u64);

                    let map = buffer.map_readable().map_err(|_| gst::FlowError::Error)?;
                    let data = map.as_slice().to_vec();

                    if let Ok(mut rb) = ring_buffer_clone.lock() {
                        rb.push(Packet { pts_ms, data });
                    }

                    Ok(gst::FlowSuccess::Ok)
                })
                .build(),
        );

        let state_change = pipeline.set_state(gst::State::Playing);
        if state_change.is_err() {
            set_state(
                &state,
                CaptureState::Failed("failed to start GStreamer pipeline".to_string()),
            );
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "failed to start GStreamer pipeline",
            ));
        }

        set_state(&state, CaptureState::Running);

        let bus_thread = spawn_bus_thread(
            &pipeline,
            state.clone(),
            stop_flag.clone(),
            callback_guard.clone(),
        )?;

        Ok(Self {
            pipeline,
            appsink: appsink.clone(),
            state,
            stop_flag,
            bus_thread: Some(bus_thread),
            callback_guard,
        })
    }

    pub fn is_running(&self) -> bool {
        matches!(*self.state.lock().unwrap(), CaptureState::Running)
    }

    pub fn state(&self) -> CaptureState {
        self.state.lock().unwrap().clone()
    }

    pub fn stop(&mut self) {
        self.stop_inner();
    }

    fn stop_inner(&mut self) {
        let should_stop = {
            let guard = self.state.lock().unwrap();
            !matches!(*guard, CaptureState::Stopped)
        };

        if !should_stop {
            return;
        }

        logger::info("capture", "stopping pipeline");

        // 1) Stop callbacks FIRST
        self.appsink
            .set_callbacks(gst_app::AppSinkCallbacks::builder().build());
        self.appsink.set_property("emit-signals", &false);
        self.callback_guard.store(true, Ordering::SeqCst);

        // 2) Stop bus thread
        self.stop_flag.store(true, Ordering::SeqCst);

        // 3) Give encoders a chance to flush and close cleanly
        let _ = self.pipeline.send_event(gst::event::Eos::new());

        // 4) Stop pipeline
        let _ = self.pipeline.set_state(gst::State::Null);

        // 5) Join bus thread
        if let Some(handle) = self.bus_thread.take() {
            let _ = handle.join();
        }

        set_state(&self.state, CaptureState::Stopped);
    }
}

impl Drop for GstCapture {
    fn drop(&mut self) {
        self.stop_inner();
    }
}

fn make_element(name: &str) -> io::Result<gst::Element> {
    gst::ElementFactory::make(name)
        .build()
        .map_err(|_| io::Error::new(io::ErrorKind::Other, format!("missing element {}", name)))
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

fn validate_config(config: &UserSettings) -> io::Result<()> {
    if let Some(mic_id) = &config.mic_device_id {
        if mic_id.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "microphone device id is empty",
            ));
        }
    }

    Ok(())
}

fn set_state(state: &Arc<Mutex<CaptureState>>, new_state: CaptureState) {
    let mut guard = state.lock().unwrap();
    logger::info("capture", format!("state: {:?} -> {:?}", *guard, new_state));
    *guard = new_state;
}

fn spawn_bus_thread(
    pipeline: &gst::Pipeline,
    state: Arc<Mutex<CaptureState>>,
    stop_flag: Arc<AtomicBool>,
    callback_guard: Arc<AtomicBool>,
) -> io::Result<std::thread::JoinHandle<()>> {
    let bus = pipeline
        .bus()
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "missing pipeline bus"))?;
    let pipeline_name = pipeline.name();

    let handle = std::thread::spawn(move || {
        while !stop_flag.load(Ordering::SeqCst) {
            let message = bus.timed_pop(gst::ClockTime::from_mseconds(200));
            let Some(message) = message else { continue };

            match message.view() {
                gst::MessageView::Error(err) => {
                    callback_guard.store(true, Ordering::SeqCst);

                    let src = err
                        .src()
                        .map(|s| s.path_string())
                        .unwrap_or_else(|| "unknown".to_string().into());
                    logger::error("gst", format!("error from {}: {}", src, err.error()));
                    if let Some(debug) = err.debug() {
                        logger::debug("gst", format!("debug: {}", debug));
                    }
                    set_state(&state, CaptureState::Failed(err.error().to_string()));
                    stop_flag.store(true, Ordering::SeqCst);
                    break;
                }
                gst::MessageView::Warning(warn) => {
                    let src = warn
                        .src()
                        .map(|s| s.path_string())
                        .unwrap_or_else(|| "unknown".to_string().into());
                    logger::warn("gst", format!("warning from {}: {}", src, warn.error()));
                    if let Some(debug) = warn.debug() {
                        logger::debug("gst", format!("debug: {}", debug));
                    }
                }
                gst::MessageView::StateChanged(state) => {
                    if message
                        .src()
                        .map(|s| s.name() == pipeline_name)
                        .unwrap_or(false)
                    {
                        logger::info(
                            "gst",
                            format!("pipeline state: {:?} -> {:?}", state.old(), state.current()),
                        );
                    }
                }
                gst::MessageView::Eos(..) => {
                    logger::error("gst", "eos");
                    set_state(&state, CaptureState::Failed("eos".to_string()));
                    break;
                }
                _ => {}
            }
        }
    });

    Ok(handle)
}

fn set_bool_property(element: &gst::Element, name: &str, value: bool) {
    if element.find_property(name).is_some() {
        element.set_property(name, &value);
    }
}
