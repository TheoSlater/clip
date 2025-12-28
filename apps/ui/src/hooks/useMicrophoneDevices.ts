import { addToast } from "@heroui/react";
import { useQuery } from "@tanstack/react-query";
import { useBackendConnectionStore } from "../state/backendConnection";
import { AudioDevice } from "../types/devices/AudioDevice";
import { useApiClient } from "./useApiClient";

export const useMicrophoneDevices = () => {
    const { get } = useApiClient();
    const status = useBackendConnectionStore((state) => state.status);

    return {
        query: useQuery({
            queryKey: ["audio", "microphones"],
            queryFn: () => get<Array<AudioDevice>>("/devices/microphones"),
            enabled: status === "connected",
            throwOnError: (error) => {
                console.error(error);
                addToast({
                    title: "Error fetching microphones",
                    description: error.message,
                    severity: "danger",
                    color: "danger",
                });
                return true;
            },
        }),
    };
};
