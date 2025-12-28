import { addToast } from "@heroui/react";
import { useQuery } from "@tanstack/react-query";
import { useBackendConnectionStore } from "../state/backendConnection";
import { VideoEncoder } from "../types/VideoEncoder";
import { useApiClient } from "./useApiClient";

export const useVideoEncoders = () => {
    const { get } = useApiClient();
    const status = useBackendConnectionStore((state) => state.status);

    return {
        query: useQuery({
            queryKey: ["video", "encoders"],
            queryFn: () => get<Array<VideoEncoder>>("/encoders/video"),
            enabled: status === "connected",
            throwOnError: (error) => {
                console.error(error);
                addToast({
                    title: "Error fetching encoders",
                    description: error.message,
                    severity: "danger",
                    color: "danger",
                });
                return true;
            },
        }),
    };
};
