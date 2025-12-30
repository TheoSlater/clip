import {
    addToast,
    Button,
    Input,
    Select,
    SelectItem,
    Slider,
    Switch,
} from "@heroui/react";
import { useQueryClient } from "@tanstack/react-query";
import { BinaryIcon, TvMinimalPlayIcon, Volume2Icon } from "lucide-react";
import { useEffect, useState } from "react";
import { useMicrophoneDevices } from "../hooks/useMicrophoneDevices";
import { useSettings } from "../hooks/useSettings";
import { useVideoDevices } from "../hooks/useVideoDevices";
import { useVideoEncoders } from "../hooks/useVideoEncoders";
import { useBackendConnectionStore } from "../state/backendConnection";
import { UserSettings } from "../types/UserSettings";
import { SectionTitle } from "./SectionTitle";

export const SettingsPanel = () => {
    const {
        query: { data: videoDevices },
    } = useVideoDevices();

    const {
        query: { data: microphoneDevices },
    } = useMicrophoneDevices();

    const {
        query: { data: encoders },
    } = useVideoEncoders();

    const {
        query: { data: settings },
        mutation: settingsMutation,
    } = useSettings();

    const queryClient = useQueryClient();
    const connectionStatus = useBackendConnectionStore((state) => state.status);

    const [form, setForm] = useState<UserSettings | null>(null);

    useEffect(() => {
        if (settings) {
            setForm(settings);
        }
    }, [settings]);
    const updateForm = <K extends keyof UserSettings>(
        key: K,
        value: UserSettings[K],
    ) => {
        setForm((prev) => (prev ? { ...prev, [key]: value } : prev));
    };

    const handleApplySettings = () => {
        if (!form) {
            return;
        }

        if (connectionStatus !== "connected") {
            addToast({
                title: "Backend offline",
                description: "Start the backend to apply settings.",
                severity: "danger",
            });
            return;
        }

        settingsMutation.mutate(form, {
            onSuccess: (data) => {
                queryClient.setQueryData(["settings"], data);
                addToast({
                    title: "Settings updated",
                    severity: "success",
                    color: "success",
                });
            },
        });
    };
    return (
        <div className="flex flex-col gap-6 p-4 bg-content1 rounded-large border-1 border-divider">
            <SectionTitle title="Video Source" Icon={TvMinimalPlayIcon}>
                <Select
                    selectedKeys={
                        form?.video_device_id ? [form.video_device_id] : []
                    }
                    onSelectionChange={(keys) => {
                        const value = Array.from(keys)[0];
                        if (typeof value === "string") {
                            updateForm("video_device_id", value);
                        }
                    }}
                    isDisabled={
                        !form ||
                        !videoDevices ||
                        connectionStatus !== "connected"
                    }
                >
                    {(videoDevices ?? []).map((device) => (
                        <SelectItem key={device.id} textValue={device.label}>
                            {`${device.label} (${device.id})`}
                        </SelectItem>
                    ))}
                </Select>
            </SectionTitle>

            <SectionTitle title="Audio Source" Icon={Volume2Icon}>
                <div className="grid grid-cols-2 gap-4">
                    <div className="flex justify-between items-center bg-default-100 rounded-medium px-3">
                        <p className="text-medium">System audio</p>
                        <Switch
                            isSelected={form?.system_audio_enabled ?? false}
                            onValueChange={(value) =>
                                updateForm("system_audio_enabled", value)
                            }
                            isDisabled={
                                !form || connectionStatus !== "connected"
                            }
                        />
                    </div>

                    <Select
                        label="Microphone"
                        selectedKeys={
                            form?.mic_device_id
                                ? [form.mic_device_id]
                                : ["none"]
                        }
                        items={microphoneDevices ?? []}
                        onSelectionChange={(keys) => {
                            const value = Array.from(keys)[0];
                            if (typeof value === "string") {
                                updateForm(
                                    "mic_device_id",
                                    value === "none" ? null : value,
                                );
                            }
                        }}
                        isDisabled={
                            !form ||
                            !microphoneDevices ||
                            connectionStatus !== "connected"
                        }
                        renderValue={(items) => {
                            return items.map((item) => (
                                <div
                                    className="flex items-center"
                                    key={item.data?.id}
                                >
                                    <p className="mr-2">{item.data?.label}</p>
                                    <p className="text-xs text-neutral-500">
                                        {item.data?.id}
                                    </p>
                                </div>
                            ));
                        }}
                    >
                        {(device) => (
                            <SelectItem
                                key={device.id}
                                textValue={device.label}
                            >
                                {device.label}
                            </SelectItem>
                        )}
                    </Select>
                </div>
            </SectionTitle>

            <SectionTitle title="Encoder Settings" Icon={BinaryIcon}>
                <div className="grid grid-cols-2 gap-4">
                    <Input
                        type="number"
                        min={1}
                        label="Framerate"
                        value={
                            typeof form?.framerate === "number"
                                ? String(form.framerate)
                                : ""
                        }
                        onValueChange={(value) => {
                            const parsed = Number(value);
                            if (!Number.isNaN(parsed)) {
                                updateForm("framerate", parsed);
                            }
                        }}
                        isDisabled={!form || connectionStatus !== "connected"}
                    />

                    <Select
                        label="Video encoder"
                        selectedKeys={
                            form?.video_encoder_id
                                ? [form.video_encoder_id]
                                : []
                        }
                        onSelectionChange={(keys) => {
                            const value = Array.from(keys)[0];
                            if (typeof value === "string") {
                                updateForm("video_encoder_id", value);
                            }
                        }}
                        isDisabled={
                            !form ||
                            !encoders ||
                            connectionStatus !== "connected"
                        }
                    >
                        {(encoders ?? []).map((encoder) => {
                            const suffixParts: string[] = [];

                            if (encoder.is_hardware) {
                                suffixParts.push("GPU");
                            }

                            if (encoder.required_memory) {
                                suffixParts.push(encoder.required_memory);
                            }

                            const suffix =
                                suffixParts.length > 0
                                    ? ` (${suffixParts.join(", ")})`
                                    : "";

                            return (
                                <SelectItem
                                    key={encoder.id}
                                    textValue={encoder.name}
                                >
                                    {`${encoder.name}${suffix}`}
                                    <p className="text-xs text-neutral-400">
                                        {encoder.id}
                                    </p>
                                </SelectItem>
                            );
                        })}
                    </Select>
                </div>

                <Slider
                    label="Bitrate (kbps)"
                    minValue={1000}
                    maxValue={20000}
                    step={1000}
                    value={form?.bitrate_kbps}
                    onChange={(value) => {
                        const parsed = Number(value);
                        if (!Number.isNaN(parsed)) {
                            updateForm("bitrate_kbps", parsed);
                        }
                    }}
                    isDisabled={!form || connectionStatus !== "connected"}
                    showSteps
                />
            </SectionTitle>

            <Button
                color="primary"
                onPress={handleApplySettings}
                isDisabled={
                    !form ||
                    settingsMutation.isPending ||
                    connectionStatus !== "connected"
                }
            >
                Apply Settings
            </Button>
        </div>
    );
};
