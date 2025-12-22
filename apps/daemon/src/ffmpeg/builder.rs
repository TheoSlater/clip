use crate::{
    ffmpeg::capture::{CaptureSource, screen_capture_args},
    state::CaptureConfig,
};

pub fn build_ffmpeg_args(config: &CaptureConfig) -> Vec<String> {
    println!("building ffmpeg args with {:?}", config);

    let source =
        CaptureSource::from_device_id(&config.video_device_id).unwrap_or(CaptureSource::Screen);

    let mut args: Vec<String> = Vec::new();

    // global
    args.extend(["-hide_banner", "-loglevel", "error"].map(String::from));

    // input options
    args.extend(["-framerate".to_string(), config.framerate.to_string()]);

    // input device
    args.extend(screen_capture_args(&source));

    // encoding
    args.extend(
        [
            "-fflags",
            "+genpts",
            "-c:v",
            "libx264",
            "-preset",
            "ultrafast",
            "-tune",
            "zerolatency",
        ]
        .map(String::from),
    );

    // output
    args.extend(["-flush_packets", "1", "-f", "mpegts", "pipe:1"].map(String::from));

    args
}
