import { useEffect, useRef } from "react";
import { axiosClient } from "../axiosClient";
import { useBackendConnectionStore } from "../state/backendConnection";

const FALLBACK_BASE_URL = "http://localhost:43123";

export const useBackendConnection = () => {
    const { setStatus, setLastError } = useBackendConnectionStore();
    const status = useBackendConnectionStore((state) => state.status);
    const lastError = useBackendConnectionStore((state) => state.lastError);
    const sourceRef = useRef<EventSource | null>(null);

    useEffect(() => {
        if (sourceRef.current) {
            return;
        }

        setStatus("connecting");

        const baseUrl = axiosClient.defaults.baseURL ?? FALLBACK_BASE_URL;
        const source = new EventSource(`${baseUrl}/events/connection`);
        sourceRef.current = source;

        source.onopen = () => {
            setStatus("connected");
            setLastError(null);
        };

        source.onerror = () => {
            setStatus("disconnected");
            setLastError("Connection lost");
            source.close();
            sourceRef.current = null;
        };

        return () => {
            source.close();
            sourceRef.current = null;
        };
    }, [setLastError, setStatus]);

    useEffect(() => {
        if (status !== "disconnected") {
            return;
        }

        const pingInterval = setInterval(async () => {
            try {
                setStatus("connecting");
                await axiosClient.get("/status");
                setStatus("connected");
            } catch (err) {
                setLastError("Connection lost");
            }
        }, 2000);

        return () => clearInterval(pingInterval);
    }, [setLastError, setStatus, status]);

    return {
        status,
        lastError,
    };
};
