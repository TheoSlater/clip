import { useQueryClient } from "@tanstack/react-query";
import { useEffect, useRef, useState } from "react";
import { axiosClient } from "../axiosClient";
import { LogEvent } from "../types/LogEvent";

const MAX_LOGS = 1000;

export const useDaemonLogs = () => {
    const queryClient = useQueryClient();
    const sourceRef = useRef<EventSource | null>(null);
    const disconnectedRef = useRef(false);
    const [logs, setLogs] = useState<Array<LogEvent>>([]);

    useEffect(() => {
        if (sourceRef.current) {
            return;
        }

        const baseUrl = axiosClient.defaults.baseURL;
        const source = new EventSource(`${baseUrl}/events/logs`);
        sourceRef.current = source;

        const onLog = (event: MessageEvent<string>) => {
            try {
                const parsed = JSON.parse(event.data) as LogEvent;
                setLogs((prev) => {
                    const next = [...prev, parsed];
                    if (next.length > MAX_LOGS) {
                        next.splice(0, next.length - MAX_LOGS);
                    }
                    return next;
                });
            } catch (err) {
                console.error(err);
            }
        };

        const onError = () => {
            if (!disconnectedRef.current) {
                disconnectedRef.current = true;
                setLogs((prev) => [
                    ...prev,
                    {
                        timestamp: new Date().toISOString(),
                        level: "warning",
                        source: "system",
                        message: "Log stream disconnected",
                    },
                ]);
            }
            source.close();
            sourceRef.current = null;
        };

        const onOpen = () => {
            disconnectedRef.current = false;
        };

        source.addEventListener("log", onLog);
        source.addEventListener("message", onLog);
        source.addEventListener("open", onOpen);
        source.addEventListener("error", onError);
        source.onmessage = onLog;

        return () => {
            source.removeEventListener("log", onLog);
            source.removeEventListener("message", onLog);
            source.removeEventListener("open", onOpen);
            source.removeEventListener("error", onError);
            source.onmessage = null;
            source.close();
            sourceRef.current = null;
        };
    }, [queryClient]);

    return logs;
};
