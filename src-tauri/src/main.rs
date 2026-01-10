#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tauri::{AppHandle, Emitter, State};
use truthbyte::{
    AppConfig, DiagnosisReport, DriveHealthStatus, DriveInspector, EventSink, ProgressUpdate,
};

const EVENT_PROGRESS: &str = "PROGRESS_UPDATE";
const EVENT_ERROR: &str = "ERROR_OCCURRED";
const EVENT_COMPLETE: &str = "DIAGNOSIS_COMPLETE";
const EVENT_CANCELLED: &str = "DIAGNOSIS_CANCELLED";

#[derive(Default)]
struct AppState {
    running: Arc<AtomicBool>,
    cancel_flag: Arc<AtomicBool>,
}

#[derive(Clone, serde::Serialize)]
struct ErrorPayload {
    message: String,
}

struct TauriEventSink {
    app: AppHandle,
}

impl EventSink for TauriEventSink {
    fn progress(&self, update: ProgressUpdate) {
        // Frontend listens to PROGRESS_UPDATE for phase/percent/speed/bytes fields.
        let _ = self.app.emit(EVENT_PROGRESS, update);
    }

    fn error(&self, message: String) {
        // Frontend listens to ERROR_OCCURRED with ErrorPayload { message }.
        let _ = self.app.emit(EVENT_ERROR, ErrorPayload { message });
    }
}

#[tauri::command]
async fn start_diagnosis(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
    limit_mb: u64,
) -> Result<(), String> {
    if state.running.swap(true, Ordering::SeqCst) {
        return Err("Diagnosis is already running.".to_string());
    }

    state.cancel_flag.store(false, Ordering::SeqCst);

    let app_handle = app.clone();
    let running = state.running.clone();
    let cancel_flag = state.cancel_flag.clone();

    tauri::async_runtime::spawn_blocking(move || {
        let inspector = DriveInspector::with_config(&path, AppConfig::default());
        let sink = TauriEventSink { app: app_handle };

        let result =
            inspector.run_write_phase_with_events(limit_mb, Some(cancel_flag.clone()), Some(&sink));

        let report = match result {
            Ok(bytes_written) => {
                if bytes_written == 0 {
                    DiagnosisReport {
                        total_capacity: 0,
                        tested_bytes: 0,
                        valid_bytes: 0,
                        error_count: 0,
                        health_score: 0.0,
                        status: DriveHealthStatus::DataLoss,
                        conclusion: "No data written; verification skipped.".to_string(),
                    }
                } else {
                    match inspector.run_verify_phase_with_events(
                        bytes_written,
                        Some(cancel_flag.clone()),
                        Some(&sink),
                    ) {
                        Ok(report) => report,
                        Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {
                            let _ = sink.app.emit(EVENT_CANCELLED, ());
                            running.store(false, Ordering::SeqCst);
                            cancel_flag.store(false, Ordering::SeqCst);
                            return;
                        }
                        Err(e) => {
                            sink.error(format!("Verify phase failed: {}", e));
                            running.store(false, Ordering::SeqCst);
                            cancel_flag.store(false, Ordering::SeqCst);
                            return;
                        }
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {
                let _ = sink.app.emit(EVENT_CANCELLED, ());
                running.store(false, Ordering::SeqCst);
                cancel_flag.store(false, Ordering::SeqCst);
                return;
            }
            Err(e) => {
                sink.error(format!("Write phase failed: {}", e));
                running.store(false, Ordering::SeqCst);
                cancel_flag.store(false, Ordering::SeqCst);
                return;
            }
        };

        // Frontend listens to DIAGNOSIS_COMPLETE with DiagnosisReport payload.
        let _ = sink.app.emit(EVENT_COMPLETE, report);
        running.store(false, Ordering::SeqCst);
        cancel_flag.store(false, Ordering::SeqCst);
    });

    Ok(())
}

#[tauri::command]
async fn stop_diagnosis(state: State<'_, AppState>) -> Result<(), String> {
    if !state.running.load(Ordering::SeqCst) {
        return Err("No active diagnosis to stop.".to_string());
    }

    state.cancel_flag.store(true, Ordering::SeqCst);
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![start_diagnosis, stop_diagnosis])
        .plugin(tauri_plugin_dialog::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
