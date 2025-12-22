import { addToast, Button, Select, SelectItem } from "@heroui/react";
import { useState } from "react";
import { useAudioDevices } from "../hooks/useAudioDevices";
import { useVideoDevices } from "../hooks/useVideoDevices";

export const Home = () => {
    const {
        query: { data: videoDevices },
        mutation: { mutate: mutateVideoDevice },
    } = useVideoDevices();

    const {
        query: { data: audioDevices },
        mutation: { mutate: mutateAudioDevice },
    } = useAudioDevices();

    const [textRes, setTextRes] = useState("{}");

    async function handleClick() {
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

    const handleChangeVideoDevice = (
        e: React.ChangeEvent<HTMLSelectElement>,
    ) => {
        // find highest framerate in capabilities
        const device = videoDevices?.find((d) => d.id === e.target.value);
        if (!device) {
            addToast({
                title: "Device not found",
                severity: "danger",
                color: "danger",
            });
            return;
        }

        mutateVideoDevice({
            deviceId: e.target.value,
            framerate: 30,
        });
    };

    const handleChangeAudioDevice = (
        e: React.ChangeEvent<HTMLSelectElement>,
    ) => {
        mutateAudioDevice({
            deviceId: e.target.value,
        });
    };

    return (
        <main className="flex flex-col items-center h-dvh gap-4 p-8 bg-black">
            {videoDevices && (
                <Select
                    label="Select Video Device"
                    onChange={handleChangeVideoDevice}
                >
                    {videoDevices.map((device) => (
                        <SelectItem key={device.id} textValue={device.label}>
                            {device.label}{" "}
                            <p className="text-xs text-gray-400">
                                ({device.id})
                            </p>
                        </SelectItem>
                    ))}
                </Select>
            )}

            {audioDevices && (
                <Select
                    label="Select Audio Device"
                    onChange={handleChangeAudioDevice}
                >
                    {audioDevices.map((device) => (
                        <SelectItem key={device.id} textValue={device.label}>
                            {device.label}{" "}
                            <p className="text-xs text-gray-400">
                                ({device.id})
                            </p>
                        </SelectItem>
                    ))}
                </Select>
            )}

            <Button color="primary" onPress={handleClick}>
                Status
            </Button>

            <Button color="primary" onPress={handlePressClip}>
                Clip
            </Button>

            <Button color="primary" onPress={handlePressListClips}>
                List Clips
            </Button>

            <Button color="danger" onPress={handlePressShutdown}>
                Shutdown Daemon
            </Button>

            <pre className="text-wrap">
                {JSON.stringify(JSON.parse(textRes || "{}"), null, 2)}
            </pre>
        </main>
    );
};
