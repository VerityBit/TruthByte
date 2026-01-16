import { useMemo, useState } from "react";
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
  const [isConfirmOpen, setIsConfirmOpen] = useState(false);

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
  const driveLabel = path || t("modal.driveUnknown");

  return (
      <div className="flex flex-col min-h-screen bg-slate-900 text-slate-50 selection:bg-sky-500/30">

        {}
        <TitleBar />

        {}
        {}
        <main
            className="flex-1 overflow-hidden flex flex-col pt-[var(--titlebar-height)]"
            style={{ height: "100vh" }}
        >

          {}
          <div className="shrink-0 h-14 border-b border-slate-800 flex items-center justify-between px-6 bg-slate-900/50 backdrop-blur-sm">
            <div className="flex items-center gap-2">
              <span className={`text-sm font-semibold ${running ? "text-sky-400" : "text-slate-400"}`}>
                {running ? "System Busy" : "System Ready"}
              </span>
              {running && <span className="flex h-2 w-2 rounded-full bg-sky-400 animate-pulse" />}
            </div>

            {}
            <div className="relative group">
              <select
                  className="appearance-none bg-transparent text-xs font-mono text-slate-500 hover:text-slate-300 uppercase cursor-pointer pr-4 focus:outline-none"
                  value={language}
                  onChange={(e) => setLanguage(e.target.value as any)}
              >
                {localeOptions.map((opt) => (
                    <option key={opt.value} value={opt.value} className="bg-slate-800 text-slate-300">
                      {opt.label}
                    </option>
                ))}
              </select>
            </div>
          </div>

          {}
          <div className="flex-1 p-6 flex flex-col overflow-hidden">
            <div className="mx-auto w-full max-w-5xl flex-1 flex flex-col transition-all duration-300">

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
                      handleStart={() => setIsConfirmOpen(true)}
                      handleStop={() => stopDiagnosis(language)}
                      setLimitMb={setLimitMb}
                  />
              )}

              {currentView === "progress" && (
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
              )}

              {currentView === "report" && (
                  <ReportView
                      t={t}
                      report={report}
                      numberFormatter={numberFormatter}
                      statusLabels={statusLabels}
                  />
              )}
            </div>
          </div>
        </main>

        {}
        {isConfirmOpen && (
            <div className="fixed inset-0 z-[100] flex items-center justify-center px-4">
              <div className="absolute inset-0 bg-slate-950/80 backdrop-blur-sm" onClick={() => setIsConfirmOpen(false)} />
              <div className="relative w-full max-w-md bg-slate-900 border border-slate-700 rounded-lg p-6 shadow-2xl">
                <h3 className="text-lg font-bold text-slate-100 mb-2">
                  {t("modal.startConfirmTitle")}
                </h3>
                <p className="text-sm text-slate-400 leading-relaxed mb-6">
                  {t("modal.startConfirmBody", { drive: driveLabel })}
                </p>
                <div className="flex gap-3 justify-end">
                  <button
                      type="button"
                      className="btn-secondary px-4 py-2 text-sm"
                      onClick={() => setIsConfirmOpen(false)}
                  >
                    {t("button.cancel")}
                  </button>
                  <button
                      type="button"
                      className="btn-primary px-4 py-2 text-sm"
                      onClick={() => {
                        setIsConfirmOpen(false);
                        startDiagnosis(limitMb, language);
                      }}
                  >
                    {t("button.startDiagnosis")}
                  </button>
                </div>
              </div>
            </div>
        )}

        {}
        <div className={`fixed bottom-6 left-1/2 -translate-x-1/2 z-[100] transition-all duration-300 ${toast ? "translate-y-0 opacity-100" : "translate-y-4 opacity-0 pointer-events-none"}`}>
          <div className="flex items-center gap-3 rounded bg-slate-800 border border-slate-600 px-4 py-3 shadow-xl">
            <span className="text-amber-400">⚠️</span>
            <span className="text-sm font-medium text-slate-200">{toast}</span>
          </div>
        </div>
      </div>
  );
}