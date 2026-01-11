import { DiagnosisReport } from "../store";

export type Translator = (
  key: string,
  vars?: Record<string, string>
) => string;

export const statusStyles: Record<DiagnosisReport["status"], string> = {
  Healthy: "border-mint-500/40 bg-mint-500/15 text-mint-400",
  FakeCapacity: "border-ember-500/50 bg-ember-500/20 text-ember-400",
  PhysicalCorruption: "border-ember-500/50 bg-ember-500/20 text-ember-400",
  DataLoss: "border-ember-500/50 bg-ember-500/20 text-ember-400"
};

export const byteFormatter = (
  formatter: Intl.NumberFormat,
  bytes: number
) => {
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

export const timeFormatter = (seconds: number | null) => {
  if (seconds === null) return "--";
  if (seconds < 60) return `${Math.round(seconds)}s`;
  const minutes = Math.floor(seconds / 60);
  const remaining = Math.round(seconds % 60);
  return `${minutes}m ${remaining}s`;
};
