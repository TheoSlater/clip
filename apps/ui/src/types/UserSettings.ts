export type UserSettings = {
    video_device_id: string;
    system_audio_enabled: boolean;
    mic_device_id?: string | null;
    video_encoder_id: string;
    framerate: number;
    bitrate_kbps: number;
};
