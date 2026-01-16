import { DiagnosisReport } from "../store";
import { byteFormatter, Translator } from "./diagnosisUtils";

type ReportViewProps = {
  t: Translator;
  report: DiagnosisReport | null;
  numberFormatter: Intl.NumberFormat;
  statusLabels: Record<DiagnosisReport["status"], string>;
};

const ReportView = ({
  t,
  report,
  numberFormatter,
  statusLabels
}: ReportViewProps) => {

  const status = report?.status || "Healthy";
  const isHealthy = status === "Healthy";
  const isFake = status === "FakeCapacity";

  const theme = isHealthy
      ? { bg: "bg-emerald-900/30", border: "border-emerald-500/50", text: "text-emerald-400", title: "VERIFIED AUTHENTIC" }
      : isFake
          ? { bg: "bg-rose-900/30", border: "border-rose-500/50", text: "text-rose-400", title: "FAKE DRIVE DETECTED" }
          : { bg: "bg-amber-900/30", border: "border-amber-500/50", text: "text-amber-400", title: "DRIVE MALFUNCTION" };

  const score = report ? Math.max(0, Math.min(100, report.health_score)) : 0;

  return (
      
      <section className="panel h-full min-h-[500px] flex flex-col relative overflow-hidden">

        {}
        <div className="h-12 border-b border-slate-700 bg-slate-800/50 flex items-center justify-between px-6 shrink-0">
        <span className="text-xs font-bold tracking-widest text-slate-400 uppercase">
          Diagnosis Report
        </span>
          <span className="text-xs font-mono text-slate-500">
          ID: {new Date().toLocaleTimeString()}
        </span>
        </div>

        <div className="flex-1 overflow-y-auto p-6 md:p-8">
          <div className="max-w-3xl mx-auto space-y-8">

            {}
            <div className={`rounded-lg border-2 p-6 md:p-8 text-center ${theme.bg} ${theme.border}`}>
              <h1 className={`text-3xl md:text-5xl font-black tracking-tight uppercase ${theme.text}`}>
                {report ? theme.title : "WAITING FOR RESULTS..."}
              </h1>
              <p className="mt-4 text-slate-300 font-medium max-w-xl mx-auto">
                {report?.conclusion || "Diagnosis is complete. Please review the statistics below."}
              </p>

              <div className="mt-6 inline-flex items-center gap-2 px-3 py-1 rounded bg-slate-900/50 border border-slate-700">
                <span className="text-xs uppercase text-slate-400 tracking-wider">Integrity Score</span>
                <span className={`font-mono font-bold ${score === 100 ? "text-emerald-400" : "text-rose-400"}`}>
                        {score}/100
                    </span>
              </div>
            </div>

            {}
            <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
              <div>
                <h3 className="text-xs font-bold uppercase text-slate-500 tracking-widest mb-4 border-b border-slate-700 pb-2">
                  Capacity Analysis
                </h3>
                <div className="space-y-4">
                  <div className="flex justify-between items-baseline">
                    <span className="text-sm text-slate-400">Total Declared</span>
                    <span className="font-mono text-slate-200">
                                {report ? byteFormatter(numberFormatter, report.total_capacity) : "--"}
                            </span>
                  </div>
                  <div className="flex justify-between items-baseline">
                    <span className="text-sm text-slate-400">Actual Valid</span>
                    <span className="font-mono text-emerald-400 font-bold">
                                {report ? byteFormatter(numberFormatter, report.valid_bytes) : "--"}
                            </span>
                  </div>
                  {isFake && (
                      <div className="flex justify-between items-baseline pt-2 border-t border-slate-700/50">
                        <span className="text-sm text-rose-400 font-bold">Oversold By</span>
                        <span className="font-mono text-rose-400 font-bold">
                                    {report ? byteFormatter(numberFormatter, Math.max(0, report.total_capacity - report.valid_bytes)) : "--"}
                                </span>
                      </div>
                  )}
                </div>
              </div>

              <div>
                <h3 className="text-xs font-bold uppercase text-slate-500 tracking-widest mb-4 border-b border-slate-700 pb-2">
                  Error Statistics
                </h3>
                <div className="space-y-4">
                  <div className="flex justify-between items-baseline">
                    <span className="text-sm text-slate-400">Data Corrupted</span>
                    <span className={`font-mono ${report?.error_count ? "text-rose-400 font-bold" : "text-slate-200"}`}>
                                {report ? report.error_count : 0} Files / Blocks
                            </span>
                  </div>
                  <div className="flex justify-between items-baseline">
                    <span className="text-sm text-slate-400">Tested Region</span>
                    <span className="font-mono text-slate-200">
                                {report ? byteFormatter(numberFormatter, report.tested_bytes) : "--"}
                            </span>
                  </div>
                </div>
              </div>
            </div>

          </div>
        </div>

        {}
        <div className="p-4 border-t border-slate-700 bg-slate-800/30 flex justify-end gap-3 shrink-0">
          <button className="btn-secondary px-6 py-2 text-sm">
            Close
          </button>
          <button className="btn-primary px-6 py-2 text-sm">
            Export Report
          </button>
        </div>
      </section>
  );
};

export default ReportView;
