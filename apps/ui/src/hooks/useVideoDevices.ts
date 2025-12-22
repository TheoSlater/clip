import { addToast } from "@heroui/react";
import { useMutation, useQuery } from "@tanstack/react-query";
import { CaptureConfig } from "../types/CaptureConfig";
import { VideoDevice } from "../types/devices/VideoDevice";
import { useApiClient } from "./useApiClient";

export const useVideoDevices = () => {
    const { get, post } = useApiClient();

    return {
        query: useQuery({
            queryKey: ["video", "devices"],
            queryFn: () => {
                return get<Array<VideoDevice>>("/devices/video");
            },
            throwOnError: (error) => {
                console.error(error);
                addToast({
                    title: "Error fetching video devices",
                    description: error.message,
                    severity: "danger",
                });
                return true;
            },
        }),
        mutation: useMutation({
            mutationFn: ({
                deviceId,
                framerate,
            }: {
                deviceId: string;
                framerate: number;
            }) => {
                return post<undefined, CaptureConfig>("/config/capture", {
                    video_device_id: deviceId,
                    framerate,
                });
            },
        }),
    };
};
