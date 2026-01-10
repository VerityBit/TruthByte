use std::fmt;
use std::io::{self, Write};
use std::path::Path;

use crate::config::AppConfig;
use crate::core_logic::{DiagnosisReport, DriveHealthStatus};
use crate::io_controller::DriveInspector;

pub struct RunOutcome {
    pub bytes_written: u64,
    pub report: DiagnosisReport,
}

#[derive(Debug)]
pub enum RunError {
    Write(io::Error),
    Verify(io::Error),
}

impl fmt::Display for RunError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RunError::Write(err) => write!(f, "write phase error: {}", err),
            RunError::Verify(err) => write!(f, "verify phase error: {}", err),
        }
    }
}

impl std::error::Error for RunError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            RunError::Write(err) => Some(err),
            RunError::Verify(err) => Some(err),
        }
    }
}

pub fn run_write_verify(
    file_path: &str,
    limit_mb: u64,
    config: AppConfig,
) -> Result<RunOutcome, RunError> {
    let inspector = DriveInspector::with_config(file_path, config);
    let bytes_written = inspector
        .run_write_phase(limit_mb)
        .map_err(RunError::Write)?;
    if bytes_written == 0 {
        let report = DiagnosisReport {
            total_capacity: 0,
            tested_bytes: 0,
            valid_bytes: 0,
            error_count: 0,
            health_score: 0.0,
            status: DriveHealthStatus::DataLoss,
            conclusion: "No data written; verification skipped.".to_string(),
        };
        return Ok(RunOutcome {
            bytes_written,
            report,
        });
    }

    let report = inspector
        .run_verify_phase(bytes_written)
        .map_err(RunError::Verify)?;
    Ok(RunOutcome {
        bytes_written,
        report,
    })
}

pub fn print_diagnostic_summary(report: &DiagnosisReport) {
    let tested_mb = report.tested_bytes as f64 / (1024.0 * 1024.0);
    let valid_mb = report.valid_bytes as f64 / (1024.0 * 1024.0);
    let total_mb = report.total_capacity as f64 / (1024.0 * 1024.0);
    let status_label = match report.status {
        DriveHealthStatus::Healthy => "Healthy",
        DriveHealthStatus::FakeCapacity => "FakeCapacity",
        DriveHealthStatus::PhysicalCorruption => "PhysicalCorruption",
        DriveHealthStatus::DataLoss => "DataLoss",
    };

    println!("========================================");
    println!("TruthByte Diagnostic Summary");
    println!("Health Score : {:.1} / 100.0", report.health_score);
    println!("Tested/Valid : {:.1} / {:.1} MB", tested_mb, valid_mb);
    println!("Total Target : {:.1} MB", total_mb);
    println!("Status       : {}", status_label);
    println!("Conclusion   : {}", report.conclusion);
    println!("========================================");
}

pub fn run_cli(args: &[String]) -> i32 {
    if args.len() < 2 {
        println!("TruthByte - USB write/verify tool.");
        println!("Usage: {} <file_path> [size_limit_mb] [--force]", args[0]);
        println!("Example: {} /Volumes/USB/test.dat 1024 --force", args[0]);
        println!("Note: size_limit_mb is optional; 0 means no limit.");
        return 0;
    }

    let file_path = &args[1];
    let mut limit_arg: Option<&str> = None;
    let mut force = false;

    for arg in args.iter().skip(2) {
        if arg == "--force" || arg == "-f" {
            force = true;
            continue;
        }

        if limit_arg.is_none() {
            limit_arg = Some(arg.as_str());
        } else {
            eprintln!("[ERROR] Unexpected argument: {}", arg);
            return 2;
        }
    }

    let limit_mb: u64 = match limit_arg {
        Some(value) => match value.parse() {
            Ok(parsed) => parsed,
            Err(_) => {
                eprintln!("[ERROR] Invalid size_limit_mb: {}", value);
                return 2;
            }
        },
        None => 0,
    };

    let path = Path::new(file_path);
    if path.is_dir() {
        eprintln!("[ERROR] Target path is a directory: {}", file_path);
        return 2;
    }

    if path.exists() && !force {
        print!(
            "[WARN] {} exists and will be overwritten. Continue? [y/N]: ",
            file_path
        );
        if io::stdout().flush().is_err() {
            eprintln!("[ERROR] Failed to prompt for confirmation.");
            return 2;
        }
        let mut response = String::new();
        if io::stdin().read_line(&mut response).is_err() {
            eprintln!("[ERROR] Failed to read confirmation.");
            return 2;
        }
        let response = response.trim().to_ascii_lowercase();
        if response != "y" && response != "yes" {
            println!("[INFO] Aborted by user.");
            return 1;
        }
    }

    match run_write_verify(file_path, limit_mb, AppConfig::default()) {
        Ok(outcome) => {
            if outcome.bytes_written == 0 {
                println!("[ERROR] No data written; verify phase skipped.");
                return 2;
            }

            print_diagnostic_summary(&outcome.report);
            if outcome.report.status == DriveHealthStatus::Healthy {
                0
            } else {
                1
            }
        }
        Err(RunError::Write(e)) => {
            println!("[ERROR] Write phase failed: {}", e);
            2
        }
        Err(RunError::Verify(e)) => {
            println!("[ERROR] Verify phase failed: {}", e);
            2
        }
    }
}
