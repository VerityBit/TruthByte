import { useEffect, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { save } from "@tauri-apps/plugin-dialog";
import {
  DiagnosisReport,
  ProgressUpdate,
  useDiagnosisStore
} from "./store";

const statusStyles: Record<DiagnosisReport["status"], string> = {
  Healthy: "border-mint-500/40 bg-mint-500/15 text-mint-400",
  FakeCapacity: "border-ember-500/50 bg-ember-500/20 text-ember-400",
  PhysicalCorruption: "border-ember-500/50 bg-ember-500/20 text-ember-400",
  DataLoss: "border-ember-500/50 bg-ember-500/20 text-ember-400"
};

const statusLabels: Record<DiagnosisReport["status"], string> = {
  Healthy: "Healthy",
  FakeCapacity: "Fake capacity",
  PhysicalCorruption: "Physical corruption",
  DataLoss: "Data loss"
};

const numberFormatter = new Intl.NumberFormat("en-US", {
  maximumFractionDigits: 1
});

const byteFormatter = (bytes: number) => {
  if (bytes <= 0) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB"];
  let index = 0;
  let value = bytes;
  while (value >= 1024 && index < units.length - 1) {
    value /= 1024;
    index += 1;
  }
  return `${numberFormatter.format(value)} ${units[index]}`;
};

const timeFormatter = (seconds: number | null) => {
  if (seconds === null) return "--";
  if (seconds < 60) return `${Math.round(seconds)}s`;
  const minutes = Math.floor(seconds / 60);
  const remaining = Math.round(seconds % 60);
  return `${minutes}m ${remaining}s`;
};

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
    setPath,
    setLimitMb,
    setRunning,
    setStatus,
    updateProgress,
    setReport,
    setToast,
    reset
  } = useDiagnosisStore();

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
          setToast("Diagnosis cancelled by user.");
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
  }, [setReport, setRunning, setStatus, setToast, updateProgress]);

  useEffect(() => {
    if (!toast) return;
    const timer = window.setTimeout(() => setToast(null), 4000);
    return () => window.clearTimeout(timer);
  }, [toast, setToast]);

  const handlePickPath = async () => {
    const selection = await save({
      title: "Select target file",
      defaultPath: "truthbyte.bin"
    });

    if (typeof selection === "string") {
      setPath(selection);
    }
  };

  const handleStart = async () => {
    if (!path) {
      setToast("Please select a target file first.");
      return;
    }

    const parsedLimit = limitMb.trim() === "" ? 0 : Number(limitMb);
    if (Number.isNaN(parsedLimit) || parsedLimit < 0) {
      setToast("Limit must be a valid number (MB).");
      return;
    }

    reset();
    setRunning(true);
    setStatus("running");

    try {
      await invoke("start_diagnosis", { path, limit_mb: parsedLimit });
    } catch (error) {
      setRunning(false);
      setStatus("error");
      setToast(
        error instanceof Error ? error.message : "Failed to start diagnosis."
      );
    }
  };

  const handleStop = async () => {
    try {
      await invoke("stop_diagnosis");
    } catch (error) {
      setToast(
        error instanceof Error ? error.message : "No active task to stop."
      );
    }
  };

  const etaSeconds = useMemo(() => {
    if (!running || speedMbps <= 0 || totalBytes === 0) return null;
    const processed = phase === "verify" ? bytesVerified : bytesWritten;
    const remaining = Math.max(totalBytes - processed, 0);
    return remaining / (1024 * 1024) / speedMbps;
  }, [bytesVerified, bytesWritten, phase, running, speedMbps, totalBytes]);

  const displayProgress = totalBytes > 0 ? Math.min(progress, 100) : 0;
  const progressBarClass =
    status === "cancelled"
      ? "bg-ink-500"
      : "bg-gradient-to-r from-mint-500 via-mint-400 to-emerald-200";

  return (
    <div className="app-shell min-h-screen px-6 py-8 text-ink-50">
      <div className="mx-auto flex max-w-6xl flex-col gap-8">
        <header className="flex flex-col gap-2">
          <p className="text-sm uppercase tracking-[0.3em] text-ink-300">
            TruthByte Diagnostics
          </p>
          <h1 className="text-4xl font-semibold">Drive Integrity Console</h1>
          <p className="max-w-2xl text-base text-ink-300">
            Validate removable drives with a deterministic write/verify routine
            and capture a trusted health report.
          </p>
        </header>

        <div className="grid gap-6 lg:grid-cols-[1.05fr_1fr]">
          <section className="rounded-3xl border border-ink-700/70 bg-ink-800/60 p-6 shadow-soft">
            <div className="flex items-center justify-between">
              <h2 className="text-xl font-semibold">Configuration</h2>
              <span
                className={`rounded-full border px-3 py-1 text-xs ${
                  running
                    ? "border-mint-500/40 bg-mint-500/10 text-mint-300"
                    : status === "cancelled"
                    ? "border-ember-500/40 bg-ember-500/10 text-ember-300"
                    : status === "error"
                    ? "border-ember-500/40 bg-ember-500/10 text-ember-300"
                    : status === "completed"
                    ? "border-mint-500/40 bg-mint-500/10 text-mint-300"
                    : "border-ink-600 text-ink-300"
                }`}
              >
                {running
                  ? "Running"
                  : status === "cancelled"
                  ? "Cancelled"
                  : status === "error"
                  ? "Error"
                  : status === "completed"
                  ? "Completed"
                  : "Idle"}
              </span>
            </div>

            <div className="mt-6 flex flex-col gap-4">
              <div className="flex flex-col gap-2">
                <label className="text-sm text-ink-300">Target file</label>
                <div className="flex flex-col gap-2 sm:flex-row">
                  <input
                    className="w-full rounded-2xl border border-ink-700 bg-ink-900/80 px-4 py-3 text-sm text-ink-50 outline-none transition focus:border-mint-500"
                    placeholder="Select a target file on the drive"
                    value={path}
                    onChange={(event) => setPath(event.target.value)}
                  />
                  <button
                    className="rounded-2xl border border-ink-600 bg-ink-700/50 px-4 py-3 text-sm font-medium transition hover:border-mint-400 hover:text-mint-200"
                    onClick={handlePickPath}
                    type="button"
                  >
                    Browse
                  </button>
                </div>
              </div>

              <div className="flex flex-col gap-2">
                <label className="text-sm text-ink-300">
                  Capacity limit (MB)
                </label>
                <input
                  className="w-full rounded-2xl border border-ink-700 bg-ink-900/80 px-4 py-3 text-sm text-ink-50 outline-none transition focus:border-mint-500"
                  placeholder="Leave empty for full-capacity scan"
                  value={limitMb}
                  onChange={(event) => setLimitMb(event.target.value)}
                />
              </div>

              <div className="flex flex-col gap-3 pt-2 sm:flex-row">
                <button
                  className="flex-1 rounded-2xl bg-mint-500 px-5 py-3 text-sm font-semibold text-ink-900 shadow-glow transition hover:bg-mint-400 disabled:cursor-not-allowed disabled:bg-ink-600"
                  onClick={handleStart}
                  disabled={running}
                >
                  Start diagnosis
                </button>
                <button
                  className="flex-1 rounded-2xl border border-ember-500/60 px-5 py-3 text-sm font-semibold text-ember-300 transition hover:border-ember-400 hover:text-ember-200 disabled:cursor-not-allowed disabled:border-ink-700 disabled:text-ink-600"
                  onClick={handleStop}
                  disabled={!running}
                >
                  Stop
                </button>
              </div>
            </div>
          </section>

          <section className="rounded-3xl border border-ink-700/70 bg-ink-800/60 p-6 shadow-soft">
            <h2 className="text-xl font-semibold">Live dashboard</h2>
            {status === "cancelled" ? (
              <div className="mt-4 rounded-2xl border border-ember-500/50 bg-ember-500/10 px-4 py-3 text-sm text-ember-200">
                Diagnosis cancelled. You can adjust the configuration and run again.
              </div>
            ) : null}
            <div className="mt-6 flex flex-col gap-6">
              <div className="rounded-2xl border border-ink-700/60 bg-ink-900/70 p-4">
                <div className="flex items-center justify-between text-sm text-ink-300">
                  <span>Progress</span>
                  <span className="font-mono text-xs">
                    {phase ? phase.toUpperCase() : "STANDBY"}
                  </span>
                </div>
                <div className="mt-3 h-3 w-full rounded-full bg-ink-700/70">
                  <div
                    className={`h-3 rounded-full transition-all ${progressBarClass}`}
                    style={{ width: `${displayProgress}%` }}
                  />
                </div>
                <div className="mt-3 flex flex-wrap items-center justify-between gap-2 text-sm">
                  <span className="text-ink-200">
                    {totalBytes > 0
                      ? `${numberFormatter.format(displayProgress)}%`
                      : "Unlimited scan"}
                  </span>
                  <span className="font-mono text-xs text-ink-400">
                    {byteFormatter(bytesWritten)} written · {byteFormatter(bytesVerified)} verified
                  </span>
                </div>
              </div>

              <div className="grid gap-4 sm:grid-cols-3">
                <div className="rounded-2xl border border-ink-700/60 bg-ink-900/70 p-4">
                  <p className="text-xs uppercase tracking-[0.2em] text-ink-400">
                    Speed
                  </p>
                  <p className="mt-3 text-2xl font-semibold">
                    {numberFormatter.format(speedMbps)}
                    <span className="text-base font-normal text-ink-300">
                      MB/s
                    </span>
                  </p>
                </div>
                <div className="rounded-2xl border border-ink-700/60 bg-ink-900/70 p-4">
                  <p className="text-xs uppercase tracking-[0.2em] text-ink-400">
                    ETA
                  </p>
                  <p className="mt-3 text-2xl font-semibold">
                    {timeFormatter(etaSeconds)}
                  </p>
                </div>
                <div className="rounded-2xl border border-ink-700/60 bg-ink-900/70 p-4">
                  <p className="text-xs uppercase tracking-[0.2em] text-ink-400">
                    Target
                  </p>
                  <p className="mt-3 text-lg font-semibold text-ink-100">
                    {totalBytes > 0
                      ? byteFormatter(totalBytes)
                      : "Full capacity"}
                  </p>
                </div>
              </div>
            </div>
          </section>
        </div>

        <section className="rounded-3xl border border-ink-700/70 bg-ink-800/60 p-6 shadow-soft">
          <div className="flex items-center justify-between">
            <h2 className="text-xl font-semibold">Diagnosis report</h2>
            {report ? (
              <span
                className={`rounded-full border px-3 py-1 text-xs ${
                  statusStyles[report.status]
                }`}
              >
                {statusLabels[report.status]}
              </span>
            ) : (
              <span className="text-xs text-ink-400">Awaiting results</span>
            )}
          </div>

          <div className="mt-6 grid gap-6 lg:grid-cols-[1.1fr_1fr]">
            <div className="rounded-2xl border border-ink-700/60 bg-ink-900/70 p-5">
              <p className="text-xs uppercase tracking-[0.2em] text-ink-400">
                Health score
              </p>
              <p className="mt-4 text-4xl font-semibold">
                {report
                  ? numberFormatter.format(report.health_score)
                  : "--"}
                <span className="text-lg font-normal text-ink-300">/100</span>
              </p>
              <p className="mt-4 text-sm text-ink-300">
                {report
                  ? report.conclusion
                  : "Run a diagnosis to generate a full integrity report."}
              </p>
            </div>

            <div className="rounded-2xl border border-ink-700/60 bg-ink-900/70 p-5">
              <p className="text-xs uppercase tracking-[0.2em] text-ink-400">
                Error sample analysis
              </p>
              <div className="mt-4 space-y-2 text-sm text-ink-200">
                <p>
                  Tested bytes: {report ? byteFormatter(report.tested_bytes) : "--"}
                </p>
                <p>
                  Valid bytes: {report ? byteFormatter(report.valid_bytes) : "--"}
                </p>
                <p>
                  Error count: {report ? report.error_count : "--"}
                </p>
                <p>
                  Total capacity: {report ? byteFormatter(report.total_capacity) : "--"}
                </p>
              </div>
              <p className="mt-5 text-xs uppercase tracking-[0.2em] text-ink-400">
                Final conclusion
              </p>
              <p className="mt-2 text-sm text-ink-300">
                {report ? statusLabels[report.status] : "Pending"}
              </p>
            </div>
          </div>
        </section>
      </div>

      {toast ? (
        <div className="fixed bottom-6 right-6 max-w-sm rounded-2xl border border-ember-500/60 bg-ink-900/90 px-4 py-3 text-sm text-ember-200 shadow-soft">
          {toast}
        </div>
      ) : null}
    </div>
  );
}
