import { useState } from "react";
import { Translator } from "./diagnosisUtils";

type SetupViewProps = {
  t: Translator;
  running: boolean;
  status: string;
  statusLabel: string;
  path: string;
  limitMb: string;
  handlePickPath: () => Promise<void>;
  handleStart: () => Promise<void>;
  handleStop: () => Promise<void>;
  setPath: (value: string) => void;
  setLimitMb: (value: string) => void;
};

const SetupView = ({
  t,
  running,
  status,
  statusLabel,
  path,
  limitMb,
  handlePickPath,
  handleStart,
  handleStop,
  setPath,
  setLimitMb
}: SetupViewProps) => {
  const [isDragActive, setIsDragActive] = useState(false);

  return (
    <section className="panel p-8">
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-semibold">{t("section.configuration")}</h2>
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

      <div className="mt-10 flex min-h-[58vh] flex-col items-center justify-center gap-6 text-center">
        <button
          className={`drop-zone group w-full max-w-3xl rounded-[28px] border border-dashed px-8 py-12 ${
            isDragActive ? "drop-zone--active" : ""
          }`}
          onClick={handlePickPath}
          onDragEnter={() => setIsDragActive(true)}
          onDragLeave={() => setIsDragActive(false)}
          onDragOver={(event) => {
            event.preventDefault();
            setIsDragActive(true);
          }}
          onDrop={(event) => {
            event.preventDefault();
            setIsDragActive(false);
          }}
          type="button"
        >
          <p className="text-lg uppercase tracking-[0.35em] text-ink-400">
            {t("dialog.selectTargetTitle")}
          </p>
          <p className="mt-4 text-3xl font-semibold text-ink-50">
            {t("placeholder.targetFile")}
          </p>
          <p className="mt-3 break-all text-sm text-ink-300">
            {path ? path : t("label.targetFile")}
          </p>
        </button>

        <button
          className="btn-primary rounded-2xl px-10 py-4 text-base font-semibold"
          onClick={handleStart}
          disabled={running}
        >
          {t("button.startDiagnosis")}
        </button>
      </div>

      <div className="mt-10">
        <details className="panel-card px-5 py-4 text-sm text-ink-300">
          <summary className="cursor-pointer text-xs uppercase tracking-[0.2em] text-ink-400">
            {t("section.configuration")}
          </summary>
          <div className="mt-4 flex flex-col gap-4">
            <div className="flex flex-col gap-2">
              <label className="text-sm text-ink-300">
                {t("label.targetFile")}
              </label>
              <input
                className="field-input w-full px-4 py-3 text-sm text-ink-50 outline-none"
                placeholder={t("placeholder.targetFile")}
                value={path}
                onChange={(event) => setPath(event.target.value)}
              />
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
                className="btn-danger-outline flex-1 rounded-2xl px-5 py-3 text-sm font-semibold"
                onClick={handleStop}
                disabled={!running}
              >
                {t("button.stop")}
              </button>
            </div>
          </div>
        </details>
      </div>
    </section>
  );
};

export default SetupView;
