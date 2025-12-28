import { addToast } from "@heroui/react";
import { useQuery } from "@tanstack/react-query";
import { useBackendConnectionStore } from "../state/backendConnection";
import { VideoDevice } from "../types/devices/VideoDevice";
import { useApiClient } from "./useApiClient";

export const useVideoDevices = () => {
    const { get } = useApiClient();
    const status = useBackendConnectionStore((state) => state.status);

    return {
        query: useQuery({
            queryKey: ["video", "devices"],
            queryFn: () => {
                return get<Array<VideoDevice>>("/devices/video");
            },
            enabled: status === "connected",
            throwOnError: (error) => {
                console.error(error);
                addToast({
                    title: "Error fetching video devices",
                    description: error.message,
                    severity: "danger",
                    color: "danger",
                });
                return true;
            },
        }),
    };
};
