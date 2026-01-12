#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use sysinfo::Disks;
use tauri::{AppHandle, Emitter, State};
use truthbyte::{AppConfig, DiagnosisReport, DriveInspector, EventSink, ProgressUpdate};

mod i18n;
use crate::i18n::{localize_conclusion, localize_error, localize_report_conclusion, Locale};

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

#[derive(serde::Serialize)]
struct SystemDisk {
    name: String,
    mount_point: String,
    total_space: u64,
    available_space: u64,
    is_removable: bool,
}

struct TauriEventSink {
    app: AppHandle,
    locale: Locale,
}

impl EventSink for TauriEventSink {
    fn progress(&self, update: ProgressUpdate) {
        let _ = self.app.emit(EVENT_PROGRESS, update);
    }

    fn error(&self, message: String) {
        let localized = localize_error(self.locale, &message);
        let _ = self.app.emit(EVENT_ERROR, ErrorPayload { message: localized });
    }
}

#[tauri::command]
fn get_system_disks() -> Vec<SystemDisk> {
    let disks = Disks::new_with_refreshed_list();
    disks
        .list()
        .iter()
        .map(|disk| SystemDisk {
            name: disk.name().to_string_lossy().into_owned(),
            mount_point: disk.mount_point().to_string_lossy().into_owned(),
            total_space: disk.total_space(),
            available_space: disk.available_space(),
            is_removable: disk.is_removable(),
        })
        .collect()
}

#[tauri::command]
async fn start_diagnosis(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
    limit_mb: u64,
    locale: String,
) -> Result<(), String> {
    let locale = Locale::from_tag(&locale);
    if state.running.swap(true, Ordering::SeqCst) {
        return Err(localize_error(locale, "Diagnosis is already running."));
    }

    state.cancel_flag.store(false, Ordering::SeqCst);

    let app_handle = app.clone();
    let running = state.running.clone();
    let cancel_flag = state.cancel_flag.clone();

    tauri::async_runtime::spawn_blocking(move || {
        let inspector = DriveInspector::with_config(&path, AppConfig::default());
        let sink = TauriEventSink {
            app: app_handle,
            locale,
        };

        let result =
            inspector.run_write_phase_with_events(limit_mb, Some(cancel_flag.clone()), Some(&sink));

        let mut report = match result {
            Ok(bytes_written) => {
                if bytes_written == 0 {
                    DiagnosisReport {
                        total_capacity: 0,
                        tested_bytes: 0,
                        valid_bytes: 0,
                        error_count: 0,
                        health_score: 0.0,
                        status: truthbyte::DriveHealthStatus::DataLoss,
                        conclusion: localize_conclusion(
                            locale,
                            "No data written; verification skipped.",
                        ),
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

        if report.total_capacity > 0 {
            report.conclusion = localize_report_conclusion(locale, &report);
        }

        let _ = sink.app.emit(EVENT_COMPLETE, report);
        running.store(false, Ordering::SeqCst);
        cancel_flag.store(false, Ordering::SeqCst);
    });

    Ok(())
}

#[tauri::command]
async fn stop_diagnosis(state: State<'_, AppState>, locale: String) -> Result<(), String> {
    let locale = Locale::from_tag(&locale);
    if !state.running.load(Ordering::SeqCst) {
        return Err(localize_error(locale, "No active diagnosis to stop."));
    }

    state.cancel_flag.store(true, Ordering::SeqCst);
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            start_diagnosis,
            stop_diagnosis,
            get_system_disks
        ])
        .plugin(tauri_plugin_dialog::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
