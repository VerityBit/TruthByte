import { useEffect, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import { join } from "@tauri-apps/api/path";
import {
  DiagnosisReport,
  ProgressUpdate,
  useDiagnosisStore
} from "./store";
import { createTranslator, localeOptions } from "./i18n/index";

const statusStyles: Record<DiagnosisReport["status"], string> = {
  Healthy: "border-mint-500/40 bg-mint-500/15 text-mint-400",
  FakeCapacity: "border-ember-500/50 bg-ember-500/20 text-ember-400",
  PhysicalCorruption: "border-ember-500/50 bg-ember-500/20 text-ember-400",
  DataLoss: "border-ember-500/50 bg-ember-500/20 text-ember-400"
};

const byteFormatter = (formatter: Intl.NumberFormat, bytes: number) => {
  if (bytes <= 0) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB"];
  let index = 0;
  let value = bytes;
  while (value >= 1024 && index < units.length - 1) {
    value /= 1024;
    index += 1;
  }
  return `${formatter.format(value)} ${units[index]}`;
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
      const target = await join(pickedPath, "truthbyte.bin");
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
  const progressBarClass =
    status === "cancelled"
      ? "bg-ink-500"
      : "bg-gradient-to-r from-mint-500 via-mint-400 to-emerald-200";
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

  return (
    <div className="app-shell min-h-screen px-6 py-8 text-ink-50">
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

        <div className="grid gap-6 lg:grid-cols-[1.05fr_1fr]">
          <section className="panel p-6">
            <div className="flex items-center justify-between">
              <h2 className="text-xl font-semibold">
                {t("section.configuration")}
              </h2>
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
                {statusLabel}
              </span>
            </div>

            <div className="mt-6 flex flex-col gap-4">
              <div className="flex flex-col gap-2">
                <label className="text-sm text-ink-300">
                  {t("label.targetFile")}
                </label>
                <div className="flex flex-col gap-2 sm:flex-row">
                  <input
                    className="field-input w-full px-4 py-3 text-sm text-ink-50 outline-none"
                    placeholder={t("placeholder.targetFile")}
                    value={path}
                    onChange={(event) => setPath(event.target.value)}
                  />
                  <button
                    className="btn-secondary rounded-2xl px-4 py-3 text-sm font-medium"
                    onClick={handlePickPath}
                    type="button"
                  >
                    {t("button.browse")}
                  </button>
                </div>
              </div>

              <div className="flex flex-col gap-2">
                <label className="text-sm text-ink-300">
                  {t("label.capacityLimit")}
                </label>
                <input
                  className="field-input w-full px-4 py-3 text-sm text-ink-50 outline-none"
                  placeholder={t("placeholder.capacityLimit")}
                  value={limitMb}
                  onChange={(event) => setLimitMb(event.target.value)}
                />
              </div>

              <div className="flex flex-col gap-3 pt-2 sm:flex-row">
                <button
                  className="btn-primary flex-1 rounded-2xl px-5 py-3 text-sm font-semibold"
                  onClick={handleStart}
                  disabled={running}
                >
                  {t("button.startDiagnosis")}
                </button>
                <button
                  className="btn-danger-outline flex-1 rounded-2xl px-5 py-3 text-sm font-semibold"
                  onClick={handleStop}
                  disabled={!running}
                >
                  {t("button.stop")}
                </button>
              </div>
            </div>
          </section>

          <section className="panel p-6">
            <h2 className="text-xl font-semibold">
              {t("section.liveDashboard")}
            </h2>
            {status === "cancelled" ? (
              <div className="mt-4 rounded-2xl border border-ember-500/50 bg-ember-500/10 px-4 py-3 text-sm text-ember-200">
                {t("banner.cancelled")}
              </div>
            ) : null}
            <div className="mt-6 flex flex-col gap-6">
              <div className="panel-card p-4">
                <div className="flex items-center justify-between text-sm text-ink-300">
                  <span>{t("progress.title")}</span>
                  <span className="font-mono text-xs">
                    {phase
                      ? t(`phase.${phase}`).toUpperCase()
                      : t("phase.standby").toUpperCase()}
                  </span>
                </div>
                <div className="progress-track mt-3 h-3 w-full rounded-full">
                  <div
                    className={`h-3 rounded-full transition-all ${progressBarClass}`}
                    style={{ width: `${displayProgress}%` }}
                  />
                </div>
                <div className="mt-3 flex flex-wrap items-center justify-between gap-2 text-sm">
                  <span className="text-ink-200">
                    {totalBytes > 0
                      ? `${numberFormatter.format(displayProgress)}%`
                      : t("progress.unlimited")}
                  </span>
                  <span className="font-mono text-xs text-ink-400">
                    {t("progress.detail", {
                      written: byteFormatter(numberFormatter, bytesWritten),
                      verified: byteFormatter(numberFormatter, bytesVerified)
                    })}
                  </span>
                </div>
              </div>

              <div className="grid gap-4 sm:grid-cols-3">
                <div className="metric-card p-4">
                  <p className="text-xs uppercase tracking-[0.2em] text-ink-400">
                    {t("metric.speed")}
                  </p>
                  <p className="mt-3 text-2xl font-semibold">
                    {numberFormatter.format(speedMbps)}
                    <span className="text-base font-normal text-ink-300">
                      {t("metric.unit.mbps")}
                    </span>
                  </p>
                </div>
                <div className="metric-card p-4">
                  <p className="text-xs uppercase tracking-[0.2em] text-ink-400">
                    {t("metric.eta")}
                  </p>
                  <p className="mt-3 text-2xl font-semibold">
                    {timeFormatter(etaSeconds)}
                  </p>
                </div>
                <div className="metric-card p-4">
                  <p className="text-xs uppercase tracking-[0.2em] text-ink-400">
                    {t("metric.target")}
                  </p>
                  <p className="mt-3 text-lg font-semibold text-ink-100">
                    {totalBytes > 0
                      ? byteFormatter(numberFormatter, totalBytes)
                      : t("metric.fullCapacity")}
                  </p>
                </div>
              </div>
            </div>
          </section>
        </div>

        <section className="panel p-6">
          <div className="flex items-center justify-between">
            <h2 className="text-xl font-semibold">
              {t("section.diagnosisReport")}
            </h2>
            {report ? (
              <span
                className={`rounded-full border px-3 py-1 text-xs ${
                  statusStyles[report.status]
                }`}
              >
                {statusLabels[report.status]}
              </span>
            ) : (
              <span className="text-xs text-ink-400">
                {t("status.awaitingResults")}
              </span>
            )}
          </div>

          <div className="mt-6 grid gap-6 lg:grid-cols-[1.1fr_1fr]">
            <div className="panel-card p-5">
              <p className="text-xs uppercase tracking-[0.2em] text-ink-400">
                {t("report.healthScore")}
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
                  : t("report.placeholder")}
              </p>
            </div>

            <div className="panel-card p-5">
              <p className="text-xs uppercase tracking-[0.2em] text-ink-400">
                {t("report.errorSampleAnalysis")}
              </p>
              <div className="mt-4 space-y-2 text-sm text-ink-200">
                <p>
                  {t("report.testedBytes")}:{" "}
                  {report
                    ? byteFormatter(numberFormatter, report.tested_bytes)
                    : "--"}
                </p>
                <p>
                  {t("report.validBytes")}:{" "}
                  {report
                    ? byteFormatter(numberFormatter, report.valid_bytes)
                    : "--"}
                </p>
                <p>
                  {t("report.errorCount")}: {report ? report.error_count : "--"}
                </p>
                <p>
                  {t("report.totalCapacity")}:{" "}
                  {report
                    ? byteFormatter(numberFormatter, report.total_capacity)
                    : "--"}
                </p>
              </div>
              <p className="mt-5 text-xs uppercase tracking-[0.2em] text-ink-400">
                {t("report.finalConclusion")}
              </p>
              <p className="mt-2 text-sm text-ink-300">
                {report ? statusLabels[report.status] : t("status.pending")}
              </p>
            </div>
          </div>
        </section>
      </div>

      {toast ? (
        <div className="toast fixed bottom-6 right-6 max-w-sm rounded-2xl px-4 py-3 text-sm text-ember-200">
          {toast}
        </div>
      ) : null}
    </div>
  );
}
