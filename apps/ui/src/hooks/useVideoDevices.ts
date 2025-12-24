import { addToast } from "@heroui/react";
import { useQuery } from "@tanstack/react-query";
import { VideoDevice } from "../types/devices/VideoDevice";
import { useApiClient } from "./useApiClient";

export const useVideoDevices = () => {
    const { get } = useApiClient();

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
    };
};
