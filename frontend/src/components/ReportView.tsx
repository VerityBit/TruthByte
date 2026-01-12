import { DiagnosisReport } from "../store";
import { byteFormatter, statusStyles, Translator } from "./diagnosisUtils";

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
  const isFakeCapacity = report?.status === "FakeCapacity";
  const glowClass = report
    ? report.status === "Healthy"
      ? "report-glow report-glow--good"
      : isFakeCapacity
      ? "report-glow report-glow--critical"
      : "report-glow report-glow--bad"
    : "report-glow";
  const score = report ? Math.max(0, Math.min(100, report.health_score)) : 0;
  const gaugeProgress = score / 100;
  const testedBytes = report ? report.tested_bytes : 0;
  const invalidBytes = report ? Math.max(report.tested_bytes - report.valid_bytes, 0) : 0;
  const badRatio = testedBytes > 0 ? invalidBytes / testedBytes : 0;
  const badCells = Math.min(100, Math.max(0, Math.round(badRatio * 100)));
  const gridCells = Array.from({ length: 100 }, (_, index) => index < badCells);
  const headline = report
    ? report.status === "FakeCapacity"
      ? t("report.alert.fakeCapacity")
      : report.status === "PhysicalCorruption"
      ? t("report.alert.physicalCorruption")
      : report.status === "DataLoss"
      ? t("report.alert.dataLoss")
      : t("report.alert.healthy")
    : t("status.awaitingResults");

  return (
    <section className={`panel p-8 ${isFakeCapacity ? "report-panel--critical" : ""}`}>
      <div className={glowClass} />
      <div className="relative z-10">
        <div className="flex flex-col gap-4 lg:flex-row lg:items-end lg:justify-between">
          <div>
            <p className="text-xs uppercase tracking-[0.3em] text-ink-400">
              {t("section.diagnosisReport")}
            </p>
            <h2 className="mt-3 text-3xl font-semibold text-ink-50">
              {headline}
            </h2>
          </div>
          {report ? (
            <span
              className={`rounded-full border px-3 py-1 text-xs ${
                statusStyles[report.status]
              }`}
            >
              {statusLabels[report.status]}
            </span>
          ) : null}
        </div>

        {isFakeCapacity ? (
          <div className="mt-6 rounded-2xl border border-ember-500/60 bg-ember-500/15 px-5 py-4">
            <p className="text-sm uppercase tracking-[0.35em] text-ember-200">
              {t("report.returnRecommendationEyebrow")}
            </p>
            <p className="mt-3 text-3xl font-semibold text-ember-100">
              {t("report.returnRecommendation")}
            </p>
          </div>
        ) : null}

        <div className="mt-10 grid gap-6 lg:grid-cols-[1.3fr_1fr]">
          <div className="panel-card p-6">
            <div className="flex flex-col gap-6 lg:flex-row lg:items-center lg:gap-10">
              <div className="gauge">
                <svg viewBox="0 0 220 120" className="gauge__svg">
                  <defs>
                    <linearGradient id="gaugeGradient" x1="0%" y1="0%" x2="100%" y2="0%">
                      <stop offset="0%" stopColor="#f87171" />
                      <stop offset="55%" stopColor="#facc15" />
                      <stop offset="100%" stopColor="#34d399" />
                    </linearGradient>
                  </defs>
                  <path
                    className="gauge__track"
                    d="M10 110 A100 100 0 0 1 210 110"
                  />
                  <path
                    className="gauge__value"
                    d="M10 110 A100 100 0 0 1 210 110"
                    style={{ strokeDashoffset: `${(1 - gaugeProgress) * 314}` }}
                  />
                </svg>
                <div className="gauge__label">
                  <p className="text-xs uppercase tracking-[0.25em] text-ink-400">
                    {t("report.healthScore")}
                  </p>
                  <p className="mt-3 text-5xl font-semibold text-ink-50">
                    {report ? numberFormatter.format(score) : "--"}
                    <span className="text-lg font-normal text-ink-300">/100</span>
                  </p>
                </div>
              </div>
              <p className="text-base text-ink-200">
                {report ? report.conclusion : t("report.placeholder")}
              </p>
            </div>
          </div>

          <div className="panel-card p-6">
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
            <div className="mt-6">
              <p className="text-xs uppercase tracking-[0.2em] text-ink-400">
                {t("report.blockMap")}
              </p>
              <div className="block-grid mt-3">
                {gridCells.map((isBad, index) => (
                  <span
                    key={`${index}`}
                    className={`block-cell ${isBad ? "block-cell--bad" : "block-cell--good"}`}
                  />
                ))}
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
};

export default ReportView;
