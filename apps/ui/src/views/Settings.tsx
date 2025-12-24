import { addToast, Button, Input, Select, SelectItem } from "@heroui/react";
import { useQueryClient } from "@tanstack/react-query";
import { ArrowLeftIcon } from "lucide-react";
import { useEffect, useState } from "react";
import { Link } from "react-router";
import { useAudioDevices } from "../hooks/useAudioDevices";
import { useSettings } from "../hooks/useSettings";
import { useVideoDevices } from "../hooks/useVideoDevices";
import { useVideoEncoders } from "../hooks/useVideoEncoders";
import { UserSettings } from "../types/UserSettings";

export const Settings = () => {
    const {
        query: { data: videoDevices },
    } = useVideoDevices();

    const {
        query: { data: audioDevices },
    } = useAudioDevices();

    const {
        query: { data: encoders },
    } = useVideoEncoders();

    const {
        query: { data: settings },
        mutation: settingsMutation,
    } = useSettings();

    const queryClient = useQueryClient();

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

        settingsMutation.mutate(form, {
            onSuccess: (data) => {
                queryClient.setQueryData(["settings"], data);
                addToast({
                    title: "Settings updated",
                    severity: "success",
                });
            },
        });
    };
    return (
        <section className="flex flex-col gap-4 p-4">
            <div className="flex gap-4">
                <Link to="/">
                    <Button isIconOnly variant="light" size="sm">
                        <ArrowLeftIcon />
                    </Button>
                </Link>
                <h2 className="text-lg font-semibold">Settings</h2>
            </div>

            <Select
                label="Video device"
                selectedKeys={
                    form?.video_device_id ? [form.video_device_id] : []
                }
                onSelectionChange={(keys) => {
                    const value = Array.from(keys)[0];
                    if (typeof value === "string") {
                        updateForm("video_device_id", value);
                    }
                }}
                isDisabled={!form || !videoDevices}
                classNames={{
                    trigger: "bg-neutral-900",
                }}
            >
                {(videoDevices ?? []).map((device) => (
                    <SelectItem key={device.id} textValue={device.label}>
                        {`${device.label} (${device.id})`}
                    </SelectItem>
                ))}
            </Select>

            <Select
                label="Audio device"
                selectedKeys={
                    form?.audio_device_id ? [form.audio_device_id] : []
                }
                onSelectionChange={(keys) => {
                    const value = Array.from(keys)[0];
                    if (typeof value === "string") {
                        updateForm("audio_device_id", value);
                    }
                }}
                isDisabled={!form || !audioDevices}
                classNames={{
                    trigger: "bg-neutral-900",
                }}
            >
                {(audioDevices ?? []).map((device) => (
                    <SelectItem key={device.id} textValue={device.label}>
                        {`${device.label} (${device.id})`}
                    </SelectItem>
                ))}
            </Select>

            <Select
                label="Video encoder"
                selectedKeys={
                    form?.video_encoder_id ? [form.video_encoder_id] : []
                }
                onSelectionChange={(keys) => {
                    const value = Array.from(keys)[0];
                    if (typeof value === "string") {
                        updateForm("video_encoder_id", value);
                    }
                }}
                isDisabled={!form || !encoders}
                classNames={{
                    trigger: "bg-neutral-900",
                }}
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
                        <SelectItem key={encoder.id} textValue={encoder.name}>
                            {`${encoder.name}${suffix}`}
                        </SelectItem>
                    );
                })}
            </Select>

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
                    isDisabled={!form}
                    classNames={{
                        inputWrapper: "bg-neutral-900",
                    }}
                />

                <Input
                    type="number"
                    min={1}
                    label="Bitrate (kbps)"
                    value={
                        typeof form?.bitrate_kbps === "number"
                            ? String(form.bitrate_kbps)
                            : ""
                    }
                    onValueChange={(value) => {
                        const parsed = Number(value);
                        if (!Number.isNaN(parsed)) {
                            updateForm("bitrate_kbps", parsed);
                        }
                    }}
                    isDisabled={!form}
                    classNames={{
                        inputWrapper: "bg-neutral-900",
                    }}
                />
            </div>

            <Button
                color="primary"
                onPress={handleApplySettings}
                isDisabled={!form || settingsMutation.isPending}
            >
                Apply Settings
            </Button>
        </section>
    );
};
