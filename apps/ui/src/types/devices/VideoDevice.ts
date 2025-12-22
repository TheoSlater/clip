import { VideoDeviceKind } from "./VideoDeviceKind";

export type VideoDevice = {
    id: string;
    label: string;
    kind: VideoDeviceKind;
};
