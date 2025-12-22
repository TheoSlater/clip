#[derive(Debug, Clone)]
pub enum CaptureSource {
    Screen,
    // Window { title: String },
    Monitor { index: u32 },
}

#[cfg(target_os = "windows")]
pub fn screen_capture_args(source: &CaptureSource) -> Vec<&'static str> {
    match source {
        CaptureSource::Screen => vec![
            "-f",
            "gdigrab",
            "-framerate",
            "60",
            "-draw_mouse",
            "1",
            "-i",
            "desktop",
        ],

        CaptureSource::Monitor { index } => {
            let offset = match index {
                0 => "0,0",
                1 => "1920,0", // example — we’ll improve this later
                _ => "0,0",
            };

            vec![
                "-f",
                "gdigrab",
                "-framerate",
                "60",
                "-offset_x",
                offset.split(',').next().unwrap(),
                "-offset_y",
                offset.split(',').nth(1).unwrap(),
                "-video_size",
                "1920x1080",
                "-draw_mouse",
                "1",
                "-i",
                "desktop",
            ]
        }
    }
}
