import { addToast } from "@heroui/react";
import { useMutation, useQuery } from "@tanstack/react-query";
import { useBackendConnectionStore } from "../state/backendConnection";
import { UserSettings } from "../types/UserSettings";
import { useApiClient } from "./useApiClient";

export const useSettings = () => {
    const { get, post } = useApiClient();
    const status = useBackendConnectionStore((state) => state.status);

    return {
        query: useQuery({
            queryKey: ["settings"],
            queryFn: () => get<UserSettings>("/settings"),
            enabled: status === "connected",
            throwOnError: (error) => {
                console.error(error);
                addToast({
                    title: "Error fetching settings",
                    description: error.message,
                    severity: "danger",
                    color: "danger",
                });
                return true;
            },
        }),
        mutation: useMutation({
            mutationFn: (settings: UserSettings) =>
                post<UserSettings, UserSettings>("/settings", settings),
            onError: (error) => {
                console.error(error);
                addToast({
                    title: "Error updating settings",
                    description: error.message,
                    severity: "danger",
                    color: "danger",
                });
            },
        }),
    };
};
