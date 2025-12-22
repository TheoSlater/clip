import { Button } from "@heroui/react";
import { useState } from "react";
import "./globals.css";

function App() {
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

    return (
        <main className="flex flex-col items-center h-dvh gap-4 p-8 bg-black">
            <h1>Welcome to Tauri + React</h1>

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
                {JSON.stringify(JSON.parse(textRes), null, 2)}
            </pre>
        </main>
    );
}

export default App;
