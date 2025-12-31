import { HeroUIProvider } from "@heroui/react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";

await invoke("start_capture_service");

listen("capture-log", (event) => {
    console.log("[capture-service]", event.payload);
});

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
    <React.StrictMode>
        <HeroUIProvider>
            <App />
        </HeroUIProvider>
    </React.StrictMode>,
);
