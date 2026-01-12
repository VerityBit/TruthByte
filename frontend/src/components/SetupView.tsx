import { DiskInfo } from "../store";
import { Translator } from "./diagnosisUtils";

type SetupViewProps = {
  t: Translator;
  running: boolean;
  statusLabel: string;
  path: string;
  limitMb: string;
  disks: DiskInfo[];
  isScanning: boolean;
  handleScan: () => Promise<void>;
  handleSelectDisk: (disk: DiskInfo) => Promise<void>;
  handleStart: () => void;
  handleStop: () => Promise<void>;
  setLimitMb: (value: string) => void;
};

const SetupView = ({
                     t,
                     running,
                     statusLabel,
                     path,
                     limitMb,
                     disks,
                     isScanning,
                     handleScan,
                     handleSelectDisk,
                     handleStart,
                     handleStop,
                     setLimitMb
                   }: SetupViewProps) => {
  const limitOptions = [
    {
      id: "quick",
      value: "1024",
      title: t("limit.quickTitle"),
      detail: t("limit.quickDesc", { size: "1 GB", time: "~10s" })
    },
    {
      id: "deep",
      value: "deep",
      title: t("limit.deepTitle"),
      detail: t("limit.deepDesc", { size: "10 GB", percent: "10%" })
    },
    {
      id: "full",
      value: "0",
      title: t("limit.fullTitle"),
      detail: t("limit.fullDesc")
    }
  ];

  const formatBytes = (bytes: number) => {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB", "TB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + " " + sizes[i];
  };

  return (
      <section className="panel animate-enter relative overflow-hidden p-8">
        <div className="flex items-center justify-between">
          <h2 className="text-xl font-semibold tracking-tight">{t("section.configuration")}</h2>
          <div className="flex gap-3">
            <button
                onClick={handleScan}
                disabled={isScanning || running}
                className="flex items-center gap-2 rounded-lg bg-ink-800/50 px-3 py-1 text-xs font-medium text-ink-300 transition-colors hover:bg-ink-700 hover:text-ink-100 disabled:opacity-50"
            >
              <svg className={`h-3 w-3 ${isScanning ? "animate-spin" : ""}`} fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
              </svg>
              {isScanning ? "Scanning..." : "Refresh"}
            </button>
            <span
                className={`rounded-full border px-3 py-1 text-xs font-medium transition-colors ${
                    running
                        ? "border-mint-500/40 bg-mint-500/10 text-mint-300 shadow-[0_0_10px_rgba(52,211,153,0.2)]"
                        : "border-ink-600 bg-ink-800/50 text-ink-300"
                }`}
            >
            {statusLabel}
            </span>
          </div>
        </div>

        <div className="mt-6 flex flex-col gap-6 lg:flex-row">
          {/* Left Column: Disk Grid */}
          <div className="flex-1">
            <label className="mb-3 block text-xs font-semibold uppercase tracking-wider text-ink-400">
              {t("label.targetFile") || "Select Drive"}
            </label>

            {disks.length === 0 ? (
                <div className="flex h-64 flex-col items-center justify-center rounded-3xl border border-dashed border-ink-700 bg-ink-800/30 text-ink-400">
                  <p className="mb-2">No drives found</p>
                  <button onClick={handleScan} className="text-sm text-sky-400 hover:underline">Retry Scan</button>
                </div>
            ) : (
                <div className="grid max-h-[400px] grid-cols-1 gap-3 overflow-y-auto pr-2 custom-scrollbar sm:grid-cols-2">
                  {disks.map((disk) => {
                    const isSelected = path.startsWith(disk.mount_point);
                    const usagePercent = ((disk.total_space - disk.available_space) / disk.total_space) * 100;

                    return (
                        <button
                            key={disk.mount_point}
                            onClick={() => handleSelectDisk(disk)}
                            disabled={running}
                            className={`group relative flex flex-col items-start gap-3 rounded-2xl border p-4 text-left transition-all ${
                                isSelected
                                    ? "border-mint-500/50 bg-mint-500/10 shadow-[0_0_20px_rgba(16,185,129,0.1)] ring-1 ring-mint-500/20"
                                    : "border-ink-700 bg-ink-800/40 hover:border-ink-500 hover:bg-ink-800/80"
                            }`}
                        >
                          <div className="flex w-full items-start justify-between">
                            <div className="flex items-center gap-3">
                              <div className={`flex h-10 w-10 items-center justify-center rounded-xl ${isSelected ? "bg-mint-500/20 text-mint-400" : "bg-ink-700 text-ink-400"}`}>
                                {disk.is_removable ? (
                                    <svg className="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M12 18h.01M8 21h8a2 2 0 002-2V5a2 2 0 00-2-2H8a2 2 0 00-2 2v14a2 2 0 002 2z" />
                                    </svg>
                                ) : (
                                    <svg className="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2m-2-4h.01M17 16h.01" />
                                    </svg>
                                )}
                              </div>
                              <div>
                                <p className={`font-medium ${isSelected ? "text-ink-50" : "text-ink-200"}`}>
                                  {disk.name || "Local Disk"}
                                </p>
                                <p className="font-mono text-xs text-ink-400">{disk.mount_point}</p>
                              </div>
                            </div>
                            {isSelected && (
                                <div className="h-2 w-2 rounded-full bg-mint-400 shadow-[0_0_8px_#34d399]" />
                            )}
                          </div>

                          <div className="w-full space-y-1.5">
                            <div className="flex justify-between text-[10px] font-medium uppercase tracking-wider text-ink-400">
                              <span>Used</span>
                              <span>{formatBytes(disk.available_space)} Free</span>
                            </div>
                            <div className="h-1.5 w-full overflow-hidden rounded-full bg-ink-900">
                              <div
                                  className={`h-full rounded-full transition-all duration-500 ${isSelected ? "bg-mint-500" : "bg-ink-500"}`}
                                  style={{ width: `${usagePercent}%` }}
                              />
                            </div>
                            <div className="text-right text-[10px] text-ink-500">
                              Total: {formatBytes(disk.total_space)}
                            </div>
                          </div>
                        </button>
                    );
                  })}
                </div>
            )}
          </div>

          {/* Right Column: Config */}
          <div className="flex w-full flex-col justify-between gap-4 rounded-3xl bg-ink-800/30 p-6 ring-1 ring-white/5 lg:w-72">
            <div className="space-y-4">
              <div className="space-y-2">
                <label className="text-xs font-semibold uppercase tracking-wider text-ink-400">
                  {t("label.scanMode")}
                </label>
                <div className="space-y-3">
                  {limitOptions.map((option) => {
                    const isSelected = limitMb === option.value;

                    return (
                      <label
                        key={option.id}
                        className={`flex cursor-pointer items-start gap-3 rounded-2xl border px-4 py-3 transition-all ${
                          isSelected
                            ? "border-sky-500/50 bg-sky-500/10 shadow-[0_0_18px_rgba(56,189,248,0.12)]"
                            : "border-ink-700 bg-ink-900/40 hover:border-ink-500 hover:bg-ink-900/70"
                        } ${running ? "cursor-not-allowed opacity-60" : ""}`}
                      >
                        <input
                          className="sr-only"
                          type="radio"
                          name="capacityLimit"
                          value={option.value}
                          checked={isSelected}
                          onChange={() => setLimitMb(option.value)}
                          disabled={running}
                        />
                        <span
                          className={`mt-1 flex h-4 w-4 items-center justify-center rounded-full border ${
                            isSelected ? "border-sky-400 bg-sky-400" : "border-ink-500"
                          }`}
                        >
                          {isSelected && <span className="h-1.5 w-1.5 rounded-full bg-ink-900" />}
                        </span>
                        <div>
                          <p className="text-sm font-semibold text-ink-50">{option.title}</p>
                          <p className="text-[11px] text-ink-400">{option.detail}</p>
                        </div>
                      </label>
                    );
                  })}
                </div>
              </div>
            </div>

            <div className="rounded-2xl border border-ember-500/40 bg-ember-500/10 px-4 py-2 text-[11px] font-semibold text-ember-200">
              {t("setup.safetyPromise")}
            </div>

            <div className="flex gap-3 pt-4">
              <button
                  className="btn-primary relative flex-1 overflow-hidden rounded-xl py-4 text-sm font-semibold uppercase tracking-wide disabled:opacity-50 disabled:cursor-not-allowed"
                  onClick={handleStart}
                  disabled={running || !path}
              >
                <span className="relative z-10">{t("button.startDiagnosis")}</span>
                <div className="absolute inset-0 -translate-x-full bg-gradient-to-r from-transparent via-white/20 to-transparent group-hover:animate-shimmer" />
              </button>

              {running && (
                  <button
                      className="btn-danger-outline rounded-xl px-4"
                      onClick={handleStop}
                      title={t("button.stop")}
                  >
                    <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                    </svg>
                  </button>
              )}
            </div>
          </div>
        </div>
      </section>
  );
};

export default SetupView;
