import { addToast } from "@heroui/react";
import { useQuery } from "@tanstack/react-query";
import { AudioDevice } from "../types/devices/AudioDevice";
import { useApiClient } from "./useApiClient";

export const useAudioDevices = () => {
    const { get } = useApiClient();

    return {
        query: useQuery({
            queryKey: ["audio", "devices"],
            queryFn: () => {
                return get<Array<AudioDevice>>("/devices/audio");
            },
            throwOnError: (error) => {
                console.error(error);
                addToast({
                    title: "Error fetching audio devices",
                    description: error.message,
                    severity: "danger",
                });
                return true;
            },
        }),
    };
};
