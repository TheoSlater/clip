import { Button, Divider } from "@heroui/react";
import { invoke } from "@tauri-apps/api/core";
import { openPath } from "@tauri-apps/plugin-opener";
import { useState } from "react";
import { useBackendConnectionStore } from "../state/backendConnection";

export const ControlPanel = () => {
    const connectionStatus = useBackendConnectionStore((state) => state.status);
    const [textRes, setTextRes] = useState("{}");

    async function handlePressStatus() {
        const response = await invoke("get_status");
        setTextRes(JSON.stringify(response, null, 2));
    }

    async function handlePressClip() {
        const response = await invoke("clip");
        setTextRes(JSON.stringify(response, null, 2));
    }

    async function handlePressListClips() {
        const clipsDir = await invoke<string>("get_clips_dir");
        openPath(clipsDir);
    }

    async function handlePressShutdown() {
        await invoke("stop_capture");
        setTextRes(JSON.stringify({ stopped: true }, null, 2));
    }
    return (
        <div className="flex flex-col gap-3 bg-content1 rounded-large p-4 border-1 border-divider">
            <Button
                color="primary"
                onPress={handlePressClip}
                isDisabled={connectionStatus !== "connected"}
            >
                Clip
            </Button>

            <Button
                onPress={handlePressStatus}
                isDisabled={connectionStatus !== "connected"}
            >
                Status
            </Button>

            <Button
                onPress={handlePressListClips}
                isDisabled={connectionStatus !== "connected"}
            >
                View Clips
            </Button>

            <Divider />

            <Button
                color="danger"
                onPress={handlePressShutdown}
                isDisabled={connectionStatus !== "connected"}
            >
                Stop Capture
            </Button>
        </div>
    );
};
