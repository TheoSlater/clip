import { addToast } from "@heroui/react";
import { useMutation, useQuery } from "@tanstack/react-query";
import { UserSettings } from "../types/UserSettings";
import { useApiClient } from "./useApiClient";

export const useSettings = () => {
    const { get, post } = useApiClient();

    return {
        query: useQuery({
            queryKey: ["settings"],
            queryFn: () => get<UserSettings>("/settings"),
            throwOnError: (error) => {
                console.error(error);
                addToast({
                    title: "Error fetching settings",
                    description: error.message,
                    severity: "danger",
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
                });
            },
        }),
    };
};
