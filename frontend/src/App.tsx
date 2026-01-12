import { useMemo } from "react";
import { createTranslator, localeOptions } from "./i18n/index";
import { useDiagnosisController } from "./hooks/useDiagnosisController";
import SetupView from "./components/SetupView";
import ProgressView from "./components/ProgressView";
import ReportView from "./components/ReportView";
import TitleBar from "./components/TitleBar";

export default function App() {
  const {
    path, limitMb, running, status, phase, progress, speedMbps,
    bytesWritten, bytesVerified, totalBytes, report, toast, language,
    disks, isScanningDisks,
    setLimitMb, setLanguage,
    scanDisks, selectDisk, startDiagnosis, stopDiagnosis
  } = useDiagnosisController();

  const t = useMemo(() => createTranslator(language), [language]);

  const numberFormatter = useMemo(
      () => new Intl.NumberFormat(language, { maximumFractionDigits: 1 }),
      [language]
  );

  const displayProgress = totalBytes > 0 ? Math.min(progress, 100) : 0;

  const etaSeconds = useMemo(() => {
    if (!running || speedMbps <= 0 || totalBytes === 0) return null;
    const processed = phase === "verify" ? bytesVerified : bytesWritten;
    const remaining = Math.max(totalBytes - processed, 0);
    return remaining / (1024 * 1024) / speedMbps;
  }, [bytesVerified, bytesWritten, phase, running, speedMbps, totalBytes]);

  const statusLabel = running
      ? t("status.running")
      : status === "cancelled" ? t("status.cancelled")
          : status === "error" ? t("status.error")
              : status === "completed" ? t("status.completed")
                  : t("status.idle");

  const statusLabels = {
    Healthy: t("report.status.healthy"),
    FakeCapacity: t("report.status.fakeCapacity"),
    PhysicalCorruption: t("report.status.physicalCorruption"),
    DataLoss: t("report.status.dataLoss")
  };

  const currentView = running ? "progress" : report ? "report" : "setup";
  const reportMood = report ? (report.status === "Healthy" ? "good" : "bad") : "neutral";

  return (
      <div className={`app-shell min-h-screen pb-8 pt-[calc(var(--titlebar-height)+32px)] text-ink-50 selection:bg-mint-500/30 selection:text-mint-100 report-${reportMood}`}>
        <TitleBar />

        <main className="mx-auto flex max-w-5xl flex-col gap-8 px-6 transition-all duration-500 ease-out">

          <header className="flex flex-col gap-1">
            <div className="flex flex-wrap items-center justify-between gap-4">
              <div className="flex items-center gap-3">
                <div className={`h-2 w-2 rounded-full transition-colors duration-500 ${running ? "bg-sky-400 shadow-[0_0_10px_#38bdf8]" : "bg-ink-600"}`}></div>
                <p className="font-mono text-xs uppercase tracking-[0.3em] text-ink-400 transition-colors hover:text-ink-300">
                  {t("app.title")} v1.0
                </p>
              </div>

              <div className="relative">
                <select
                    className="cursor-pointer appearance-none rounded-lg bg-ink-800/50 py-1.5 pl-3 pr-8 text-xs font-medium text-ink-300 ring-1 ring-white/5 transition-colors hover:bg-ink-800 hover:text-ink-100 focus:outline-none focus:ring-sky-500/50"
                    value={language}
                    onChange={(e) => setLanguage(e.target.value as any)}
                >
                  {localeOptions.map((opt) => (
                      <option key={opt.value} value={opt.value}>{opt.label}</option>
                  ))}
                </select>
                <div className="pointer-events-none absolute right-2.5 top-1/2 -translate-y-1/2 text-ink-500">
                  <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M6 9l6 6 6-6"/></svg>
                </div>
              </div>
            </div>

            <div className="mt-4 space-y-2">
              <h1 className="bg-gradient-to-br from-white to-ink-400 bg-clip-text text-4xl font-bold tracking-tight text-transparent">
                {t("app.heading")}
              </h1>
              <p className="max-w-2xl text-base text-ink-400">
                {t("app.subheading")}
              </p>
            </div>
          </header>

          <div className="relative min-h-[400px]">
            {currentView === "setup" && (
                <SetupView
                    t={t}
                    running={running}
                    statusLabel={statusLabel}
                    path={path}
                    limitMb={limitMb}
                    disks={disks}
                    isScanning={isScanningDisks}
                    handleScan={scanDisks}
                    handleSelectDisk={selectDisk}
                    handleStart={() => startDiagnosis(limitMb, language)}
                    handleStop={() => stopDiagnosis(language)}
                    setLimitMb={setLimitMb}
                />
            )}

            {currentView === "progress" && (
                <div className="animate-enter">
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
                      handleStop={() => stopDiagnosis(language)}
                  />
                </div>
            )}

            {currentView === "report" && (
                <div className="animate-enter">
                  <ReportView
                      t={t}
                      report={report}
                      numberFormatter={numberFormatter}
                      statusLabels={statusLabels}
                  />
                </div>
            )}
          </div>

        </main>

        <div className={`fixed bottom-8 left-1/2 z-50 -translate-x-1/2 transition-all duration-300 ${toast ? "translate-y-0 opacity-100" : "translate-y-4 opacity-0 pointer-events-none"}`}>
          <div className="flex items-center gap-3 rounded-2xl border border-ember-500/20 bg-ink-900/95 px-6 py-4 shadow-[0_8px_30px_rgba(0,0,0,0.5)] backdrop-blur-xl">
            <div className="flex h-6 w-6 items-center justify-center rounded-full bg-ember-500/10">
              <span className="text-ember-400">!</span>
            </div>
            <span className="text-sm font-medium text-ink-100">{toast}</span>
          </div>
        </div>
      </div>
  );
}