import { useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { join } from "@tauri-apps/api/path";
import { useDiagnosisStore, ProgressUpdate, DiagnosisReport, DiskInfo } from "../store";
import { Locale } from "../i18n";

export function useDiagnosisController() {
    const store = useDiagnosisStore();

    const resolveTargetPath = useCallback(async (basePath: string) => {
        return join(basePath, "truthbyte.bin");
    }, []);

    useEffect(() => {
        let mounted = true;
        let unlisteners: (() => void)[] = [];

        const setupListeners = async () => {
            const unlistenProgress = await listen<ProgressUpdate>("PROGRESS_UPDATE", (event) => {
                if (!mounted) return;
                store.updateProgress(event.payload);
            });

            const unlistenError = await listen<{ message: string }>("ERROR_OCCURRED", (event) => {
                if (!mounted) return;
                store.setToast(event.payload.message);
                store.setRunning(false);
                store.setStatus("error");
            });

            const unlistenComplete = await listen<DiagnosisReport>("DIAGNOSIS_COMPLETE", (event) => {
                if (!mounted) return;
                store.setReport(event.payload);
                store.setRunning(false);
                store.setStatus("completed");
            });

            const unlistenCancelled = await listen<void>("DIAGNOSIS_CANCELLED", () => {
                if (!mounted) return;
                store.setToast("Diagnosis cancelled by user");
                store.setRunning(false);
                store.setStatus("cancelled");
            });

            unlisteners.push(unlistenProgress, unlistenError, unlistenComplete, unlistenCancelled);
        };

        setupListeners();
        scanDisks();

        return () => {
            mounted = false;
            unlisteners.forEach((fn) => fn());
        };
    }, []);

    useEffect(() => {
        if (!store.toast) return;
        const timer = window.setTimeout(() => store.setToast(null), 4000);
        return () => window.clearTimeout(timer);
    }, [store.toast]);

    const scanDisks = async () => {
        store.setScanningDisks(true);
        try {
            const disks = await invoke<DiskInfo[]>("get_system_disks");
            store.setDisks(disks);
        } catch (err) {
            console.error("Failed to scan disks:", err);
            store.setToast("Failed to list system disks");
        } finally {
            store.setScanningDisks(false);
        }
    };

    const selectDisk = async (disk: DiskInfo) => {
        try {
            const target = await resolveTargetPath(disk.mount_point);
            store.setPath(target);
        } catch (err) {
            console.error(err);
            store.setToast("Failed to select disk");
        }
    };

    const startDiagnosis = async (limitMb: string, locale: Locale) => {
        if (!store.path) {
            store.setToast("Please select a target disk first");
            return;
        }

        let parsedLimit: number;
        if (limitMb === "deep") {
            const selectedDisk = store.disks.find((disk) =>
                store.path.startsWith(disk.mount_point)
            );
            if (selectedDisk) {
                const totalMb = selectedDisk.total_space / (1024 * 1024);
                parsedLimit = Math.max(1, Math.floor(Math.min(10240, totalMb * 0.1)));
            } else {
                parsedLimit = 10240;
            }
        } else {
            parsedLimit = limitMb.trim() === "" ? 0 : Number(limitMb);
        }
        if (Number.isNaN(parsedLimit) || parsedLimit < 0) {
            store.setToast("Invalid limit value");
            return;
        }

        store.reset();
        store.setRunning(true);
        store.setStatus("running");

        try {
            await invoke("start_diagnosis", {
                path: store.path,
                limit_mb: parsedLimit,
                locale: locale
            });
        } catch (error) {
            store.setRunning(false);
            store.setStatus("error");
            store.setToast(error instanceof Error ? error.message : "Failed to start");
        }
    };

    const stopDiagnosis = async (locale: Locale) => {
        try {
            await invoke("stop_diagnosis", { locale });
        } catch (error) {
            store.setToast(error instanceof Error ? error.message : "Failed to stop");
        }
    };

    return {
        ...store,
        scanDisks,
        selectDisk,
        startDiagnosis,
        stopDiagnosis
    };
}
