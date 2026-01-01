import { Button, Divider } from "@heroui/react";
import { invoke } from "@tauri-apps/api/core";
import { openPath } from "@tauri-apps/plugin-opener";
import { useBackendConnectionStore } from "../state/backendConnection";

export const ControlPanel = () => {
    const connectionStatus = useBackendConnectionStore((state) => state.status);

    function handlePressStatus() {
        invoke("get_status");
    }

    function handlePressClip() {
        invoke("clip");
    }

    async function handlePressListClips() {
        const clipsDir = await invoke<string>("get_clips_dir");
        openPath(clipsDir);
    }

    function handlePressShutdown() {
        invoke("stop_capture");
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
