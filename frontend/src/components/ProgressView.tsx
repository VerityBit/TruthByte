import { byteFormatter, timeFormatter, Translator } from "./diagnosisUtils";

type ProgressViewProps = {
  t: Translator;
  status: string;
  phase: string | null;
  displayProgress: number;
  numberFormatter: Intl.NumberFormat;
  bytesWritten: number;
  bytesVerified: number;
  speedMbps: number;
  etaSeconds: number | null;
  totalBytes: number;
  handleStop: () => Promise<void>;
};

const ProgressView = ({
                        t,
                        status,
                        phase,
                        displayProgress,
                        numberFormatter,
                        bytesWritten,
                        bytesVerified,
                        speedMbps,
                        etaSeconds,
                        totalBytes,
                        handleStop
                      }: ProgressViewProps) => {

  const phaseLabel = phase === "write" ? "WRITING DATA"
      : phase === "verify" ? "VERIFYING INTEGRITY"
          : "PREPARING";

  return (
      
      <section className="panel h-full min-h-[500px] flex flex-col items-center justify-center p-8 relative overflow-hidden">

        {}
        <div className="absolute top-0 left-0 right-0 h-12 border-b border-slate-700 bg-slate-800/50 flex items-center justify-between px-6">
          <div className="flex items-center gap-2">
            <div className={`w-2 h-2 rounded-full ${status === "running" ? "bg-emerald-400 animate-pulse" : "bg-slate-500"}`}></div>
            <span className="text-xs font-bold tracking-widest text-slate-400 uppercase">
                {status === "cancelled" ? "CANCELLED" : phaseLabel}
            </span>
          </div>
          <div className="text-xs font-mono text-slate-500">
            {}
          </div>
        </div>

        {}
        <div className="w-full max-w-2xl flex flex-col items-center gap-8 z-10">

          {}
          <div className="text-center">
            <h1 className="text-7xl md:text-8xl font-mono font-bold text-slate-100 tracking-tighter tabular-nums">
              {totalBytes > 0 ? numberFormatter.format(displayProgress) : "0.0"}
              <span className="text-2xl text-slate-500 ml-2">%</span>
            </h1>
          </div>

          {}
          <div className="w-full space-y-2">
            <div className="h-6 w-full bg-slate-800 rounded border border-slate-700 overflow-hidden relative">
              <div
                  className="h-full bg-sky-500 transition-all duration-300 ease-out"
                  style={{ width: `${Math.min(displayProgress, 100)}%` }}
              />
              <div className="absolute inset-0 bg-gradient-to-b from-white/5 to-transparent pointer-events-none"></div>
            </div>
            <div className="flex justify-between text-xs font-mono text-slate-400 uppercase">
              <span>Processed: {byteFormatter(numberFormatter, phase === "write" ? bytesWritten : bytesVerified)}</span>
              <span>Total: {totalBytes > 0 ? byteFormatter(numberFormatter, totalBytes) : "Calculating..."}</span>
            </div>
          </div>

          {}
          <div className="grid grid-cols-2 gap-4 w-full mt-4">
            <div className="bg-slate-800/50 border border-slate-700 p-4 rounded flex flex-col items-center">
              <span className="text-[10px] font-bold uppercase tracking-widest text-slate-500 mb-1">Current Speed</span>
              <span className="text-2xl font-mono text-sky-400 font-semibold tabular-nums">
                    {numberFormatter.format(speedMbps)} <span className="text-sm text-slate-500">MB/s</span>
                </span>
            </div>

            <div className="bg-slate-800/50 border border-slate-700 p-4 rounded flex flex-col items-center">
              <span className="text-[10px] font-bold uppercase tracking-widest text-slate-500 mb-1">Time Remaining</span>
              <span className="text-2xl font-mono text-emerald-400 font-semibold tabular-nums">
                    {etaSeconds !== null ? timeFormatter(etaSeconds) : "--:--"}
                </span>
            </div>
          </div>

          {}
          <button
              onClick={handleStop}
              className="mt-4 px-8 py-2 text-sm font-bold text-rose-400 border border-rose-900/50 hover:bg-rose-900/20 hover:text-rose-300 rounded transition-colors uppercase tracking-wider"
          >
            {status === "cancelled" ? "Return to Menu" : "Abort Operation"}
          </button>

        </div>
      </section>
  );
};

export default ProgressView;