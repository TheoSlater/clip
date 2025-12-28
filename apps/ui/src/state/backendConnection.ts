import { create } from "zustand";

export type BackendConnectionStatus =
    | "disconnected"
    | "connecting"
    | "connected";

type BackendConnectionState = {
    status: BackendConnectionStatus;
    lastError: string | null;
    setStatus: (status: BackendConnectionStatus) => void;
    setLastError: (message: string | null) => void;
};

export const useBackendConnectionStore = create<BackendConnectionState>(
    (set) => ({
        status: "disconnected",
        lastError: null,
        setStatus: (status) => set({ status }),
        setLastError: (message) => set({ lastError: message }),
    }),
);
