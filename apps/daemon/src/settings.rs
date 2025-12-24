use serde::{Deserialize, Serialize};
use std::{
    fs,
    io,
    path::{Path, PathBuf},
};

use directories::ProjectDirs;

use crate::{
    capture_devices::{AudioDevice, VideoDevice, VideoDeviceKind},
    encoders::VideoEncoderDescriptor,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettings {
    pub video_device_id: String,
    pub audio_device_id: String,
    pub video_encoder_id: String,
    pub framerate: u32,
    pub bitrate_kbps: u32,
}

pub fn settings_path() -> io::Result<PathBuf> {
    let project =
        ProjectDirs::from("com", "clip", "clip").ok_or_else(|| {
            io::Error::new(io::ErrorKind::Other, "failed to resolve config directory")
        })?;
    Ok(project.config_dir().join("settings.json"))
}

pub fn load_settings() -> io::Result<Option<UserSettings>> {
    let path = settings_path()?;
    if !path.exists() {
        return Ok(None);
    }

    let data = fs::read_to_string(&path)?;
    let settings =
        serde_json::from_str(&data).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    Ok(Some(settings))
}

pub fn save_settings(settings: &UserSettings) -> io::Result<()> {
    let path = settings_path()?;
    ensure_parent_dir(&path)?;

    let data = serde_json::to_string_pretty(settings)
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
    fs::write(path, data)?;
    Ok(())
}

pub fn default_settings(
    video_devices: &[VideoDevice],
    audio_devices: &[AudioDevice],
    encoders: &[VideoEncoderDescriptor],
) -> io::Result<UserSettings> {
    let default_video = prefer_screen_device(video_devices).ok_or_else(|| {
        io::Error::new(io::ErrorKind::Other, "no video devices available")
    })?;
    let default_encoder = prefer_hardware_encoder(encoders).ok_or_else(|| {
        io::Error::new(io::ErrorKind::Other, "no video encoders available")
    })?;

    let default_audio = audio_devices.first().ok_or_else(|| {
        io::Error::new(io::ErrorKind::Other, "no audio devices available")
    })?;

    Ok(UserSettings {
        video_device_id: default_video.id.clone(),
        audio_device_id: default_audio.id.clone(),
        video_encoder_id: default_encoder.id.clone(),
        framerate: 60,
        bitrate_kbps: 20_000,
    })
}

pub fn apply_startup_fallbacks(
    mut settings: UserSettings,
    video_devices: &[VideoDevice],
    audio_devices: &[AudioDevice],
    encoders: &[VideoEncoderDescriptor],
) -> (UserSettings, Vec<String>) {
    let mut changes = Vec::new();

    if !video_devices.iter().any(|d| d.id == settings.video_device_id) {
        if let Some(default_video) = prefer_screen_device(video_devices) {
            settings.video_device_id = default_video.id.clone();
            changes.push("video device reset to default".to_string());
        }
    }

    if !audio_devices
        .iter()
        .any(|d| d.id == settings.audio_device_id)
    {
        if let Some(default_audio) = audio_devices.first() {
            settings.audio_device_id = default_audio.id.clone();
            changes.push("audio device reset to default".to_string());
        }
    }

    if !encoders.iter().any(|e| e.id == settings.video_encoder_id) {
        if let Some(default_encoder) = prefer_hardware_encoder(encoders) {
            settings.video_encoder_id = default_encoder.id.clone();
            changes.push("video encoder reset to default".to_string());
        }
    }

    if settings.framerate == 0 {
        settings.framerate = 60;
        changes.push("framerate reset to 60".to_string());
    }

    if settings.bitrate_kbps == 0 {
        settings.bitrate_kbps = 20_000;
        changes.push("bitrate reset to 20000 kbps".to_string());
    }

    (settings, changes)
}

pub fn validate_settings(
    settings: &UserSettings,
    video_devices: &[VideoDevice],
    audio_devices: &[AudioDevice],
    encoders: &[VideoEncoderDescriptor],
) -> Result<(), String> {
    if !video_devices.iter().any(|d| d.id == settings.video_device_id) {
        return Err("selected video device is not available".to_string());
    }

    if !audio_devices
        .iter()
        .any(|d| d.id == settings.audio_device_id)
    {
        return Err("selected audio device is not available".to_string());
    }

    if !encoders.iter().any(|e| e.id == settings.video_encoder_id) {
        return Err("selected video encoder is not available".to_string());
    }

    if settings.framerate == 0 {
        return Err("framerate must be greater than zero".to_string());
    }

    if settings.bitrate_kbps == 0 {
        return Err("bitrate must be greater than zero".to_string());
    }

    Ok(())
}

fn ensure_parent_dir(path: &Path) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

fn prefer_hardware_encoder<'a>(
    encoders: &'a [VideoEncoderDescriptor],
) -> Option<&'a VideoEncoderDescriptor> {
    encoders
        .iter()
        .find(|enc| enc.is_hardware)
        .or_else(|| encoders.first())
}

fn prefer_screen_device<'a>(devices: &'a [VideoDevice]) -> Option<&'a VideoDevice> {
    devices
        .iter()
        .find(|device| matches!(device.kind, VideoDeviceKind::Screen))
        .or_else(|| devices.first())
}
