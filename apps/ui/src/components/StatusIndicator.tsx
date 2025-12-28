import { Chip, ChipProps } from "@heroui/react";
import { useBackendConnectionStore } from "../state/backendConnection";

export const StatusIndicator = () => {
    const status = useBackendConnectionStore((state) => state.status);

    const getColor = (): ChipProps["color"] => {
        switch (status) {
            case "connected":
                return "success";
            case "connecting":
                return "warning";
            case "disconnected":
                return "danger";
        }
    };

    return (
        <Chip
            size="md"
            color={getColor()}
            variant="flat"
            className="capitalize p-1"
        >
            <div className="flex items-center gap-2">
                <div className="w-2 h-2 rounded-full bg-current" />
                {status}
            </div>
        </Chip>
    );
};
