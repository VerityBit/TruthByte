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
  const ringRadius = 118;
  const ringCircumference = 2 * Math.PI * ringRadius;
  const progressOffset =
    ringCircumference - (Math.min(displayProgress, 100) / 100) * ringCircumference;
  const isIndeterminate = totalBytes <= 0;
  const ringClass =
    status === "cancelled" ? "progress-ring__value--cancelled" : "progress-ring__value";

  return (
    <section className="panel p-8">
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-semibold">{t("section.liveDashboard")}</h2>
        <span className="text-xs uppercase tracking-[0.25em] text-ink-400">
          {phase ? t(`phase.${phase}`) : t("phase.standby")}
        </span>
      </div>

      {status === "cancelled" ? (
        <div className="mt-4 rounded-2xl border border-ember-500/50 bg-ember-500/10 px-4 py-3 text-sm text-ember-200">
          {t("banner.cancelled")}
        </div>
      ) : null}

      <div className="mt-10 grid gap-10 lg:grid-cols-[1.1fr_1fr]">
        <div className="flex items-center justify-center">
          <div className="relative flex h-[320px] w-[320px] items-center justify-center">
            <div className="heartbeat-glow" />
            <svg
              className={`progress-ring ${isIndeterminate ? "progress-ring--indeterminate" : ""}`}
              width="320"
              height="320"
              viewBox="0 0 320 320"
            >
              <defs>
                <linearGradient id="ringGradient" x1="0%" y1="0%" x2="100%" y2="0%">
                  <stop offset="0%" stopColor="#38bdf8" />
                  <stop offset="55%" stopColor="#34d399" />
                  <stop offset="100%" stopColor="#a7f3d0" />
                </linearGradient>
              </defs>
              <circle
                className="progress-ring__track"
                cx="160"
                cy="160"
                r={ringRadius}
                strokeWidth="16"
              />
              <circle
                className={ringClass}
                cx="160"
                cy="160"
                r={ringRadius}
                strokeWidth="16"
                strokeDasharray={ringCircumference}
                strokeDashoffset={progressOffset}
              />
            </svg>
            <div className="absolute text-center">
              <p className="text-xs uppercase tracking-[0.3em] text-ink-400">
                {t("progress.title")}
              </p>
              <p className="mt-3 text-4xl font-semibold text-ink-50">
                {totalBytes > 0
                  ? `${numberFormatter.format(displayProgress)}%`
                  : t("progress.unlimited")}
              </p>
              <p className="mt-3 font-mono text-xs text-ink-400">
                {t("progress.detail", {
                  written: byteFormatter(numberFormatter, bytesWritten),
                  verified: byteFormatter(numberFormatter, bytesVerified)
                })}
              </p>
            </div>
          </div>
        </div>

        <div className="flex flex-col items-center justify-center gap-6 text-center">
          <div className="metric-card w-full max-w-sm px-6 py-6">
            <p className="text-xs uppercase tracking-[0.2em] text-ink-400">
              {t("metric.speed")}
            </p>
            <p className="mt-3 text-4xl font-semibold">
              {numberFormatter.format(speedMbps)}
              <span className="text-base font-normal text-ink-300">
                {t("metric.unit.mbps")}
              </span>
            </p>
          </div>
          <div className="metric-card w-full max-w-sm px-6 py-6">
            <p className="text-xs uppercase tracking-[0.2em] text-ink-400">
              {t("metric.eta")}
            </p>
            <p className="mt-3 text-3xl font-semibold">
              {timeFormatter(etaSeconds)}
            </p>
          </div>
          <div className="metric-card w-full max-w-sm px-6 py-6">
            <p className="text-xs uppercase tracking-[0.2em] text-ink-400">
              {t("metric.target")}
            </p>
            <p className="mt-3 text-2xl font-semibold text-ink-100">
              {totalBytes > 0
                ? byteFormatter(numberFormatter, totalBytes)
                : t("metric.fullCapacity")}
            </p>
          </div>
          <button
            className="btn-danger-outline mt-2 rounded-2xl px-6 py-3 text-sm font-semibold"
            onClick={handleStop}
            type="button"
          >
            {t("button.stop")}
          </button>
        </div>
      </div>
    </section>
  );
};

export default ProgressView;
