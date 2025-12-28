import { Button, Divider } from "@heroui/react";
import { useState } from "react";
import { useBackendConnectionStore } from "../state/backendConnection";

export const ControlPanel = () => {
    const connectionStatus = useBackendConnectionStore((state) => state.status);
    const [textRes, setTextRes] = useState("{}");

    async function handlePressStatus() {
        const response = await fetch("http://localhost:43123/status");
        setTextRes(await response.text());
    }

    async function handlePressClip() {
        const response = await fetch("http://localhost:43123/clip", {
            method: "POST",
        });
        setTextRes(await response.text());
    }

    async function handlePressListClips() {
        const response = await fetch("http://localhost:43123/clips");
        setTextRes(await response.text());
    }

    async function handlePressShutdown() {
        const response = await fetch("http://localhost:43123/shutdown", {
            method: "POST",
        });
        setTextRes(await response.text());
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
                Shutdown Daemon
            </Button>
        </div>
    );
};
