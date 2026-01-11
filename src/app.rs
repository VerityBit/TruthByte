use std::fmt;
use std::io::{self, Write};
use std::path::Path;

use crate::config::AppConfig;
use crate::core_logic::{DiagnosisReport, DriveHealthStatus};
use crate::i18n::cli as i18n;
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
    let locale = i18n::detect_locale();
    let status_label = i18n::status_label(locale, report.status);
    let conclusion = if report.total_capacity == 0 {
        report.conclusion.clone()
    } else {
        i18n::localize_report_conclusion(locale, report)
    };
    let summary_title = i18n::summary_title(locale);
    let health_label = i18n::health_label(locale);
    let tested_label = i18n::tested_label(locale);
    let total_label = i18n::total_label(locale);
    let status_header = i18n::status_header(locale);
    let conclusion_header = i18n::conclusion_header(locale);

    println!("========================================");
    println!("{summary_title}");
    println!("{health_label} : {:.1} / 100.0", report.health_score);
    println!("{tested_label} : {:.1} / {:.1} MB", tested_mb, valid_mb);
    println!("{total_label} : {:.1} MB", total_mb);
    println!("{status_header}       : {}", status_label);
    println!("{conclusion_header}   : {}", conclusion);
    println!("========================================");
}

pub fn run_cli(args: &[String]) -> i32 {
    let locale = i18n::detect_locale();
    if args.len() < 2 {
        println!("{}", i18n::cli_intro(locale));
        println!("{}", i18n::cli_usage(locale, &args[0]));
        println!("{}", i18n::cli_example(locale, &args[0]));
        println!("{}", i18n::cli_note(locale));
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
            let message = i18n::cli_unexpected_arg(locale);
            eprintln!("[ERROR] {}: {}", message, arg);
            return 2;
        }
    }

    let limit_mb: u64 = match limit_arg {
        Some(value) => match value.parse() {
            Ok(parsed) => parsed,
            Err(_) => {
                let message = i18n::cli_invalid_limit(locale);
                eprintln!("[ERROR] {}: {}", message, value);
                return 2;
            }
        },
        None => 0,
    };

    let path = Path::new(file_path);
    if path.is_dir() {
        let message = i18n::cli_target_is_dir(locale);
        eprintln!("[ERROR] {}: {}", message, file_path);
        return 2;
    }

    if path.exists() && !force {
        print!("{}", i18n::cli_overwrite_prompt(locale, file_path));
        if io::stdout().flush().is_err() {
            let message = i18n::cli_prompt_flush_failed(locale);
            eprintln!("[ERROR] {}", message);
            return 2;
        }
        let mut response = String::new();
        if io::stdin().read_line(&mut response).is_err() {
            let message = i18n::cli_prompt_read_failed(locale);
            eprintln!("[ERROR] {}", message);
            return 2;
        }
        let response = response.trim().to_ascii_lowercase();
        if response != "y" && response != "yes" {
            let message = i18n::cli_user_aborted(locale);
            println!("[INFO] {}", message);
            return 1;
        }
    }

    match run_write_verify(file_path, limit_mb, AppConfig::default()) {
        Ok(outcome) => {
            if outcome.bytes_written == 0 {
                let message = i18n::cli_no_data_written(locale);
                println!("[ERROR] {}", message);
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
            let message = i18n::cli_write_phase_failed(locale);
            println!("[ERROR] {}: {}", message, e);
            2
        }
        Err(RunError::Verify(e)) => {
            let message = i18n::cli_verify_phase_failed(locale);
            println!("[ERROR] {}: {}", message, e);
            2
        }
    }
}
