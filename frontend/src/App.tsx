import { useCallback, useEffect, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import { dirname, join } from "@tauri-apps/api/path";
import {
  DiagnosisReport,
  ProgressUpdate,
  useDiagnosisStore
} from "./store";
import { createTranslator, localeOptions } from "./i18n/index";
import SetupView from "./components/SetupView";
import ProgressView from "./components/ProgressView";
import ReportView from "./components/ReportView";
import TitleBar from "./components/TitleBar";

export default function App() {
  const {
    path,
    limitMb,
    running,
    status,
    phase,
    progress,
    speedMbps,
    bytesWritten,
    bytesVerified,
    totalBytes,
    report,
    toast,
    language,
    setPath,
    setLimitMb,
    setRunning,
    setStatus,
    updateProgress,
    setReport,
    setToast,
    setLanguage,
    reset
  } = useDiagnosisStore();
  const t = useMemo(() => createTranslator(language), [language]);
  const numberFormatter = useMemo(
    () => new Intl.NumberFormat(language, { maximumFractionDigits: 1 }),
    [language]
  );
  const resolveTargetPath = useCallback(async (pickedPath: string) => {
    const looksLikeFile = /[\\/][^\\/]+\.[^\\/]+$/.test(pickedPath);
    const baseDir = looksLikeFile ? await dirname(pickedPath) : pickedPath;
    return join(baseDir, "truthbyte.bin");
  }, []);

  useEffect(() => {
    let mounted = true;

    const init = async () => {
      const unlistenProgress = await listen<ProgressUpdate>(
        "PROGRESS_UPDATE",
        (event) => {
          if (!mounted) return;
          // PROGRESS_UPDATE from Rust updates progress metrics and phase state.
          updateProgress(event.payload);
        }
      );

      const unlistenError = await listen<{ message: string }>(
        "ERROR_OCCURRED",
        (event) => {
          if (!mounted) return;
          // ERROR_OCCURRED from Rust maps to toast and stops running state.
          setToast(event.payload.message);
          setRunning(false);
          setStatus("error");
        }
      );

      const unlistenComplete = await listen<DiagnosisReport>(
        "DIAGNOSIS_COMPLETE",
        (event) => {
          if (!mounted) return;
          // DIAGNOSIS_COMPLETE from Rust updates report state for the result view.
          setReport(event.payload);
          setRunning(false);
          setStatus("completed");
        }
      );

      const unlistenCancelled = await listen<void>(
        "DIAGNOSIS_CANCELLED",
        () => {
          if (!mounted) return;
          // DIAGNOSIS_CANCELLED from Rust resets running state with a toast.
          setToast(t("toast.cancelledByUser"));
          setRunning(false);
          setStatus("cancelled");
        }
      );

      return () => {
        unlistenProgress();
        unlistenError();
        unlistenComplete();
        unlistenCancelled();
      };
    };

    const cleanupPromise = init();
    return () => {
      mounted = false;
      void cleanupPromise.then((cleanup) => cleanup && cleanup());
    };
  }, [setReport, setRunning, setStatus, setToast, t, updateProgress]);

  useEffect(() => {
    let mounted = true;
    const init = async () => {
      const unlistenFileDrop = await listen<string[]>(
        "tauri://file-drop",
        async (event) => {
          if (!mounted) return;
          const [pickedPath] = event.payload ?? [];
          if (!pickedPath) return;
          const target = await resolveTargetPath(pickedPath);
          setPath(target);
        }
      );

      return () => {
        unlistenFileDrop();
      };
    };

    const cleanupPromise = init();
    return () => {
      mounted = false;
      void cleanupPromise.then((cleanup) => cleanup && cleanup());
    };
  }, [resolveTargetPath, setPath]);

  useEffect(() => {
    if (!toast) return;
    const timer = window.setTimeout(() => setToast(null), 4000);
    return () => window.clearTimeout(timer);
  }, [toast, setToast]);

  const handlePickPath = async () => {
    const selection = await open({
      title: t("dialog.selectTargetTitle"),
      directory: true,
      multiple: false
    });

    const pickedPath = Array.isArray(selection) ? selection[0] : selection;
    if (typeof pickedPath === "string") {
      const target = await resolveTargetPath(pickedPath);
      setPath(target);
    }
  };

  const handleStart = async () => {
    if (!path) {
      setToast(t("toast.selectTarget"));
      return;
    }

    const parsedLimit = limitMb.trim() === "" ? 0 : Number(limitMb);
    if (Number.isNaN(parsedLimit) || parsedLimit < 0) {
      setToast(t("toast.invalidLimit"));
      return;
    }

    reset();
    setRunning(true);
    setStatus("running");

    try {
      await invoke("start_diagnosis", {
        path,
        limit_mb: parsedLimit,
        locale: language
      });
    } catch (error) {
      setRunning(false);
      setStatus("error");
      setToast(error instanceof Error ? error.message : t("toast.startFailed"));
    }
  };

  const handleStop = async () => {
    try {
      await invoke("stop_diagnosis", { locale: language });
    } catch (error) {
      setToast(error instanceof Error ? error.message : t("toast.stopFailed"));
    }
  };

  const etaSeconds = useMemo(() => {
    if (!running || speedMbps <= 0 || totalBytes === 0) return null;
    const processed = phase === "verify" ? bytesVerified : bytesWritten;
    const remaining = Math.max(totalBytes - processed, 0);
    return remaining / (1024 * 1024) / speedMbps;
  }, [bytesVerified, bytesWritten, phase, running, speedMbps, totalBytes]);

  const displayProgress = totalBytes > 0 ? Math.min(progress, 100) : 0;
  const statusLabel = running
    ? t("status.running")
    : status === "cancelled"
    ? t("status.cancelled")
    : status === "error"
    ? t("status.error")
    : status === "completed"
    ? t("status.completed")
    : t("status.idle");
  const statusLabels: Record<DiagnosisReport["status"], string> = {
    Healthy: t("report.status.healthy"),
    FakeCapacity: t("report.status.fakeCapacity"),
    PhysicalCorruption: t("report.status.physicalCorruption"),
    DataLoss: t("report.status.dataLoss")
  };
  const view = running ? "progress" : report ? "report" : "setup";
  const reportMood = report
    ? report.status === "Healthy"
      ? "good"
      : "bad"
    : "neutral";

  return (
    <div
      className={`app-shell min-h-screen px-6 pb-8 pt-[calc(var(--titlebar-height)+32px)] text-ink-50 view-${view} report-${reportMood}`}
    >
      <TitleBar />
      <div className="mx-auto flex max-w-6xl flex-col gap-8">
        <header className="flex flex-col gap-2">
          <div className="flex flex-wrap items-center justify-between gap-4">
            <p className="text-sm uppercase tracking-[0.3em] text-ink-300">
              {t("app.title")}
            </p>
            <div className="flex items-center gap-2">
              <label className="text-xs uppercase tracking-[0.2em] text-ink-400">
                {t("label.language")}
              </label>
              <select
                className="field-input px-3 py-2 text-xs text-ink-50"
                value={language}
                onChange={(event) =>
                  setLanguage(event.target.value as typeof language)
                }
              >
                {localeOptions.map((option) => (
                  <option key={option.value} value={option.value}>
                    {option.label}
                  </option>
                ))}
              </select>
            </div>
          </div>
          <h1 className="text-4xl font-semibold">{t("app.heading")}</h1>
          <p className="max-w-2xl text-base text-ink-300">
            {t("app.subheading")}
          </p>
        </header>

        {view === "setup" ? (
          <SetupView
            t={t}
            running={running}
            status={status}
            statusLabel={statusLabel}
            path={path}
            limitMb={limitMb}
            handlePickPath={handlePickPath}
            handleStart={handleStart}
            handleStop={handleStop}
            setPath={setPath}
            setLimitMb={setLimitMb}
          />
        ) : null}

        {view === "progress" ? (
          <ProgressView
            t={t}
            status={status}
            phase={phase}
            displayProgress={displayProgress}
            numberFormatter={numberFormatter}
            bytesWritten={bytesWritten}
            bytesVerified={bytesVerified}
            speedMbps={speedMbps}
            etaSeconds={etaSeconds}
            totalBytes={totalBytes}
            handleStop={handleStop}
          />
        ) : null}

        {view === "report" ? (
          <ReportView
            t={t}
            report={report}
            numberFormatter={numberFormatter}
            statusLabels={statusLabels}
          />
        ) : null}
      </div>

      {toast ? (
        <div className="toast fixed bottom-6 right-6 max-w-sm rounded-2xl px-4 py-3 text-sm text-ember-200">
          {toast}
        </div>
      ) : null}
    </div>
  );
}
