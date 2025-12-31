import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef, useState } from "react";
import { LogEvent } from "../types/LogEvent";

const MAX_LOGS = 1000;

export const useDaemonLogs = () => {
    const unlistenRef = useRef<null | (() => void)>(null);
    const [logs, setLogs] = useState<Array<LogEvent>>([]);

    useEffect(() => {
        let active = true;

        invoke<Array<LogEvent>>("get_recent_logs")
            .then((events) => {
                if (!active || events.length === 0) {
                    return;
                }
                setLogs((prev) => {
                    const merged = [...events, ...prev];
                    if (merged.length > MAX_LOGS) {
                        merged.splice(0, merged.length - MAX_LOGS);
                    }
                    return merged;
                });
            })
            .catch((error) => {
                if (!active) {
                    return;
                }
                setLogs((prev) => [
                    ...prev,
                    {
                        timestamp: new Date().toISOString(),
                        level: "warning",
                        source: "system",
                        message:
                            error instanceof Error
                                ? error.message
                                : "Failed to load recent logs",
                    },
                ]);
            });

        listen<LogEvent>("capture-log", (event) => {
            if (!active) {
                return;
            }
            console.log("[capture]", event.payload);
            setLogs((prev) => {
                const next = [...prev, event.payload];
                if (next.length > MAX_LOGS) {
                    next.splice(0, next.length - MAX_LOGS);
                }
                return next;
            });
        })
            .then((unlisten) => {
                if (!active) {
                    unlisten();
                    return;
                }
                unlistenRef.current = unlisten;
            })
            .catch((error) => {
                if (!active) {
                    return;
                }
                setLogs((prev) => [
                    ...prev,
                    {
                        timestamp: new Date().toISOString(),
                        level: "warning",
                        source: "system",
                        message:
                            error instanceof Error
                                ? error.message
                                : "Log stream unavailable",
                    },
                ]);
            });

        return () => {
            active = false;
            if (unlistenRef.current) {
                unlistenRef.current();
                unlistenRef.current = null;
            }
        };
    }, []);

    return logs;
};
