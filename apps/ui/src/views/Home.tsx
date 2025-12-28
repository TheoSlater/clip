import { ControlPanel } from "../components/ControlPanel";
import { LogPanel } from "../components/LogPanel";
import { SettingsPanel } from "../components/SettingsPanel";
import { StatusIndicator } from "../components/StatusIndicator";

export const Home = () => {
    return (
        <main className="flex flex-col h-dvh gap-4 p-8 pt-4">
            <StatusIndicator />

            <div className="flex gap-4">
                <SettingsPanel />
                <ControlPanel />
            </div>

            <LogPanel />
        </main>
    );
};
