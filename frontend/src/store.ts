import { create } from "zustand";
import { Locale, getInitialLocale, persistLocale } from "./i18n/index";

export type ProgressPhase = "write" | "verify";

export interface ProgressUpdate {
  phase: ProgressPhase;
  percent: number;
  speed_mbps: number;
  bytes_written: number;
  bytes_verified: number;
  total_bytes: number;
}

export interface DiagnosisReport {
  total_capacity: number;
  tested_bytes: number;
  valid_bytes: number;
  error_count: number;
  health_score: number;
  status: "Healthy" | "FakeCapacity" | "PhysicalCorruption" | "DataLoss";
  conclusion: string;
}

interface DiagnosisState {
  path: string;
  limitMb: string;
  running: boolean;
  status: "idle" | "running" | "cancelled" | "error" | "completed";
  phase: ProgressPhase | null;
  progress: number;
  speedMbps: number;
  bytesWritten: number;
  bytesVerified: number;
  totalBytes: number;
  report: DiagnosisReport | null;
  toast: string | null;
  language: Locale;
  setPath: (path: string) => void;
  setLimitMb: (limitMb: string) => void;
  setRunning: (running: boolean) => void;
  setStatus: (status: DiagnosisState["status"]) => void;
  updateProgress: (update: ProgressUpdate) => void;
  setReport: (report: DiagnosisReport | null) => void;
  setToast: (message: string | null) => void;
  setLanguage: (language: Locale) => void;
  reset: () => void;
}

export const useDiagnosisStore = create<DiagnosisState>((set) => ({
  path: "",
  limitMb: "",
  running: false,
  status: "idle",
  phase: null,
  progress: 0,
  speedMbps: 0,
  bytesWritten: 0,
  bytesVerified: 0,
  totalBytes: 0,
  report: null,
  toast: null,
  language: getInitialLocale(),
  setPath: (path) => set({ path }),
  setLimitMb: (limitMb) => set({ limitMb }),
  setRunning: (running) => set({ running }),
  setStatus: (status) => set({ status }),
  updateProgress: (update) =>
    set({
      phase: update.phase,
      progress: update.percent,
      speedMbps: update.speed_mbps,
      bytesWritten: update.bytes_written,
      bytesVerified: update.bytes_verified,
      totalBytes: update.total_bytes
    }),
  setReport: (report) => set({ report }),
  setToast: (toast) => set({ toast }),
  setLanguage: (language) => {
    persistLocale(language);
    set({ language });
  },
  reset: () =>
    set({
      status: "idle",
      phase: null,
      progress: 0,
      speedMbps: 0,
      bytesWritten: 0,
      bytesVerified: 0,
      totalBytes: 0,
      report: null
    })
}));
