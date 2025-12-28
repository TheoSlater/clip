export type LogLevel = "debug" | "info" | "warning" | "error";

export type LogEvent = {
    timestamp: string;
    level: LogLevel;
    source: string;
    message: string;
};
