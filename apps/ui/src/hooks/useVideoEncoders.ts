import { addToast } from "@heroui/react";
import { useQuery } from "@tanstack/react-query";
import { VideoEncoder } from "../types/VideoEncoder";
import { useApiClient } from "./useApiClient";

export const useVideoEncoders = () => {
    const { get } = useApiClient();

    return {
        query: useQuery({
            queryKey: ["video", "encoders"],
            queryFn: () => get<Array<VideoEncoder>>("/encoders/video"),
            throwOnError: (error) => {
                console.error(error);
                addToast({
                    title: "Error fetching encoders",
                    description: error.message,
                    severity: "danger",
                });
                return true;
            },
        }),
    };
};
