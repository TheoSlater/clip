import { addToast } from "@heroui/react";
import { useMutation, useQuery } from "@tanstack/react-query";
import { AudioDevice } from "../types/devices/AudioDevice";
import { useApiClient } from "./useApiClient";

export const useAudioDevices = () => {
    const { get, post } = useApiClient();

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
        mutation: useMutation({
            mutationFn: ({ deviceId }: { deviceId: string }) => {
                return post("/config/capture", {});
            },
        }),
    };
};
