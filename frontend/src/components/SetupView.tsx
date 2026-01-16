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
        { id: "quick", value: "1024", label: t("limit.quickLabel") },
        { id: "deep", value: "deep", label: t("limit.deepLabel") },
        { id: "full", value: "0", label: t("limit.fullLabel") }
    ];

    const formatBytes = (bytes: number) => {
        if (bytes === 0) return "0 B";
        const k = 1024;
        const sizes = ["B", "KB", "MB", "GB", "TB"];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + " " + sizes[i];
    };

    return (
        <section className="panel flex flex-col h-full min-h-[500px] overflow-hidden md:flex-row">
            {}
            <div className="flex-1 flex flex-col border-b md:border-b-0 md:border-r border-slate-700 overflow-hidden">

                {}
                <div className="flex items-center justify-between p-4 border-b border-slate-700 bg-slate-800/50 shrink-0">
                    <h2 className="font-semibold text-slate-200">{t("label.targetFile") || "Select Target Drive"}</h2>
                    <button
                        onClick={handleScan}
                        disabled={isScanning || running}
                        className="btn-secondary px-3 py-1 text-xs h-8"
                    >
                        {isScanning ? "Scanning..." : "Refresh List"}
                    </button>
                </div>

                {}
                {}
                {}
                <div className="flex-1 overflow-y-auto p-2 pr-2 space-y-1 bg-slate-900 custom-scrollbar">
                    {disks.length === 0 ? (
                        <div className="flex h-full flex-col items-center justify-center text-slate-500">
                            <p>No drives detected.</p>
                        </div>
                    ) : (
                        disks.map((disk) => {
                            const isSelected = path.startsWith(disk.mount_point);
                            const usagePercent = ((disk.total_space - disk.available_space) / disk.total_space) * 100;

                            return (
                                <button
                                    key={disk.mount_point}
                                    onClick={() => handleSelectDisk(disk)}
                                    disabled={running}
                                    className={`w-full flex items-center gap-4 px-4 py-3 text-left rounded border transition-colors group ${
                                        isSelected
                                            ? "bg-sky-900/40 border-sky-500/50 ring-1 ring-sky-500/50"
                                            : "bg-slate-800 border-transparent hover:bg-slate-700 hover:border-slate-600"
                                    }`}
                                >
                                    {}
                                    <div className={`shrink-0 ${isSelected ? "text-sky-400" : "text-slate-400 group-hover:text-slate-300"}`}>
                                        <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d={disk.is_removable
                                                ? "M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4"
                                                : "M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2m-2-4h.01M17 16h.01"
                                            } />
                                        </svg>
                                    </div>

                                    {}
                                    <div className="flex-1 min-w-0 grid grid-cols-1 md:grid-cols-2 gap-x-4 gap-y-1">
                                        <div>
                                            <div className="font-semibold text-sm text-slate-100 truncate">
                                                {disk.name || "Local Disk"} <span className="text-slate-500 font-mono">({disk.mount_point})</span>
                                            </div>
                                            <div className="text-xs text-slate-400">
                                                {formatBytes(disk.available_space)} free of {formatBytes(disk.total_space)}
                                            </div>
                                        </div>

                                        {}
                                        <div className="hidden md:flex flex-col justify-center gap-1">
                                            <div className="h-2 w-full bg-slate-700 rounded-full overflow-hidden">
                                                <div
                                                    className={`h-full ${isSelected ? "bg-sky-500" : "bg-slate-500"}`}
                                                    style={{ width: `${usagePercent}%` }}
                                                />
                                            </div>
                                        </div>
                                    </div>
                                </button>
                            );
                        })
                    )}
                </div>
            </div>

            {}
            <div className="w-full md:w-64 bg-slate-800/30 p-4 flex flex-col gap-6 flex-shrink-0">

                {}
                <div className="space-y-3">
                    <label className="text-xs font-bold uppercase text-slate-400 tracking-wider">
                        {t("label.scanMode")}
                    </label>
                    <div className="flex flex-col gap-2">
                        {limitOptions.map((opt) => (
                            <label
                                key={opt.id}
                                className={`flex items-center gap-3 px-3 py-2 rounded border cursor-pointer select-none transition-colors ${
                                    limitMb === opt.value
                                        ? "bg-sky-900/30 border-sky-500/50 text-sky-100"
                                        : "bg-slate-900/50 border-slate-700 text-slate-400 hover:border-slate-500"
                                } ${running ? "opacity-50 cursor-not-allowed" : ""}`}
                            >
                                <input
                                    type="radio"
                                    name="scanMode"
                                    value={opt.value}
                                    checked={limitMb === opt.value}
                                    onChange={() => setLimitMb(opt.value)}
                                    disabled={running}
                                    className="sr-only"
                                />
                                <div className={`w-3 h-3 rounded-full border ${limitMb === opt.value ? "border-sky-400 bg-sky-400" : "border-slate-500"}`}></div>
                                <span className="text-sm font-medium">{opt.label}</span>
                            </label>
                        ))}
                    </div>
                </div>

                <div className="mt-auto space-y-3">
                    {}
                    <div className="flex items-center justify-between text-xs text-slate-400 bg-slate-900/50 p-2 rounded border border-slate-700">
                        <span>Status</span>
                        <span className={running ? "text-sky-400 font-bold" : "text-slate-300"}>{statusLabel}</span>
                    </div>

                    {}
                    <div className="flex flex-col gap-2">
                        {!running ? (
                            <button
                                onClick={handleStart}
                                disabled={!path}
                                className="btn-primary w-full py-3 text-sm"
                            >
                                {t("button.startDiagnosis")}
                            </button>
                        ) : (
                            <button
                                onClick={handleStop}
                                className="btn-danger w-full py-3 text-sm border-rose-500/50 hover:bg-rose-900/20"
                            >
                                {t("button.stop")}
                            </button>
                        )}
                    </div>
                </div>
            </div>
        </section>
    );
};

export default SetupView;
