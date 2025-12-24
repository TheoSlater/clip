use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub enum VideoDeviceKind {
    Screen,
    Camera,
}

#[derive(Debug, Clone, Serialize)]
pub struct VideoDevice {
    pub id: String,
    pub label: String,
    pub kind: VideoDeviceKind,

    #[cfg(target_os = "windows")]
    pub monitor_index: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AudioDevice {
    pub id: String,
    pub label: String,
    pub is_input: bool,
}

#[cfg(target_os = "windows")]
mod windows {
    use crate::capture_devices::{AudioDevice, VideoDevice, VideoDeviceKind};
    use windows::{
        Win32::Foundation::{BOOL, LPARAM},
        Win32::Graphics::Gdi::{
            EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFOEXW,
        },
    };

    pub fn list_video_devices() -> Vec<VideoDevice> {
        let mut devices = Vec::new();

        unsafe extern "system" fn enum_monitor(
            hmonitor: HMONITOR,
            _hdc: HDC,
            _rect: *mut windows::Win32::Foundation::RECT,
            lparam: LPARAM,
        ) -> BOOL {
            let data = unsafe { &mut *(lparam.0 as *mut Vec<VideoDevice>) };

            let mut info = MONITORINFOEXW::default();
            info.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;

            if unsafe { GetMonitorInfoW(hmonitor, &mut info.monitorInfo as *mut _ as _) } == false {
                return BOOL(1);
            }

            let label = String::from_utf16_lossy(
                &info
                    .szDevice
                    .iter()
                    .take_while(|c| **c != 0)
                    .cloned()
                    .collect::<Vec<u16>>(),
            );

            let index = data.len() as u32;

            data.push(VideoDevice {
                id: format!("screen:{}", index),
                label,
                kind: VideoDeviceKind::Screen,
                monitor_index: Some(index),
            });

            BOOL(1)
        }

        unsafe {
            EnumDisplayMonitors(
                HDC(0),
                None,
                Some(enum_monitor),
                LPARAM(&mut devices as *mut _ as isize),
            );
        }

        devices
    }

    pub fn list_audio_devices() -> Vec<AudioDevice> {
        vec![AudioDevice {
            id: "loopback".to_string(),
            label: "System Audio (Loopback)".to_string(),
            is_input: false,
        }]
    }
}

#[cfg(not(target_os = "windows"))]
mod other {
    use crate::capture_devices::{AudioDevice, VideoDevice};

    pub fn list_video_devices() -> Vec<VideoDevice> {
        Vec::new()
    }

    pub fn list_audio_devices() -> Vec<AudioDevice> {
        Vec::new()
    }
}

pub fn list_video_devices() -> Vec<VideoDevice> {
    #[cfg(target_os = "windows")]
    return windows::list_video_devices();

    #[cfg(not(target_os = "windows"))]
    return other::list_video_devices();
}

pub fn list_audio_devices() -> Vec<AudioDevice> {
    #[cfg(target_os = "windows")]
    return windows::list_audio_devices();

    #[cfg(not(target_os = "windows"))]
    return other::list_audio_devices();
}
