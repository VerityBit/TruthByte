use std::env;
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

#[derive(Clone, Copy)]
pub enum Locale {
    En,
    ZhCn,
    ZhTw,
    Ja,
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

fn parse_locale(value: &str) -> Option<Locale> {
    let locale = value.to_lowercase();
    if locale.starts_with("zh-cn") || locale.starts_with("zh_cn") || locale.starts_with("zh-hans")
    {
        return Some(Locale::ZhCn);
    }
    if locale.starts_with("zh-tw")
        || locale.starts_with("zh_tw")
        || locale.starts_with("zh-hant")
        || locale.starts_with("zh-hk")
    {
        return Some(Locale::ZhTw);
    }
    if locale.starts_with("ja") {
        return Some(Locale::Ja);
    }
    if locale.starts_with("en") {
        return Some(Locale::En);
    }
    None
}

fn detect_locale() -> Locale {
    for key in ["LC_ALL", "LC_MESSAGES", "LANG"] {
        if let Ok(value) = env::var(key) {
            if let Some(locale) = parse_locale(&value) {
                return locale;
            }
        }
    }
    Locale::En
}

fn status_label(locale: Locale, status: DriveHealthStatus) -> &'static str {
    match (locale, status) {
        (Locale::ZhCn, DriveHealthStatus::Healthy) => "健康",
        (Locale::ZhCn, DriveHealthStatus::FakeCapacity) => "虚假容量",
        (Locale::ZhCn, DriveHealthStatus::PhysicalCorruption) => "物理损坏",
        (Locale::ZhCn, DriveHealthStatus::DataLoss) => "数据丢失",
        (Locale::ZhTw, DriveHealthStatus::Healthy) => "健康",
        (Locale::ZhTw, DriveHealthStatus::FakeCapacity) => "虛假容量",
        (Locale::ZhTw, DriveHealthStatus::PhysicalCorruption) => "實體損壞",
        (Locale::ZhTw, DriveHealthStatus::DataLoss) => "資料遺失",
        (Locale::Ja, DriveHealthStatus::Healthy) => "健全",
        (Locale::Ja, DriveHealthStatus::FakeCapacity) => "偽容量",
        (Locale::Ja, DriveHealthStatus::PhysicalCorruption) => "物理破損",
        (Locale::Ja, DriveHealthStatus::DataLoss) => "データ損失",
        (_, DriveHealthStatus::Healthy) => "Healthy",
        (_, DriveHealthStatus::FakeCapacity) => "FakeCapacity",
        (_, DriveHealthStatus::PhysicalCorruption) => "PhysicalCorruption",
        (_, DriveHealthStatus::DataLoss) => "DataLoss",
    }
}

fn localize_report_conclusion(locale: Locale, report: &DiagnosisReport) -> String {
    let base = match report.status {
        DriveHealthStatus::Healthy => match locale {
            Locale::En => "No inconsistencies detected.",
            Locale::ZhCn => "未发现不一致。",
            Locale::ZhTw => "未發現不一致。",
            Locale::Ja => "不整合は検出されませんでした。",
        },
        DriveHealthStatus::FakeCapacity => match locale {
            Locale::En => "Warning: detected zero-filled data; possible fake capacity.",
            Locale::ZhCn => "警告：检测到全零数据，可能为虚假容量。",
            Locale::ZhTw => "警告：偵測到全零資料，可能為虛假容量。",
            Locale::Ja => "警告：ゼロ埋めデータを検出。偽容量の可能性があります。",
        },
        DriveHealthStatus::PhysicalCorruption => match locale {
            Locale::En => "Warning: detected random corruption or inconsistent data.",
            Locale::ZhCn => "警告：检测到随机损坏或不一致数据。",
            Locale::ZhTw => "警告：偵測到隨機損壞或不一致資料。",
            Locale::Ja => "警告：ランダムな破損または不整合なデータを検出しました。",
        },
        DriveHealthStatus::DataLoss => match locale {
            Locale::En => "Warning: read errors or missing data detected.",
            Locale::ZhCn => "警告：检测到读取错误或数据缺失。",
            Locale::ZhTw => "警告：偵測到讀取錯誤或資料遺失。",
            Locale::Ja => "警告：読み取りエラーまたはデータ欠落を検出しました。",
        },
    };

    let mut conclusion = match locale {
        Locale::En => format!("{base} Errors: {}.", report.error_count),
        Locale::ZhCn => format!("{base} 错误数：{}。", report.error_count),
        Locale::ZhTw => format!("{base} 錯誤數：{}。", report.error_count),
        Locale::Ja => format!("{base} エラー数：{}。", report.error_count),
    };

    if report.tested_bytes < report.total_capacity {
        match locale {
            Locale::En => conclusion.push_str(&format!(
                " Verification ended early after {} / {} bytes.",
                report.tested_bytes, report.total_capacity
            )),
            Locale::ZhCn => conclusion.push_str(&format!(
                " 校验提前结束，已完成 {} / {} 字节。",
                report.tested_bytes, report.total_capacity
            )),
            Locale::ZhTw => conclusion.push_str(&format!(
                " 驗證提前結束，已完成 {} / {} 位元組。",
                report.tested_bytes, report.total_capacity
            )),
            Locale::Ja => conclusion.push_str(&format!(
                " 検証は早期終了しました（{} / {} バイト）。",
                report.tested_bytes, report.total_capacity
            )),
        }
    }

    conclusion
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
    let locale = detect_locale();
    let status_label = status_label(locale, report.status);
    let conclusion = if report.total_capacity == 0 {
        report.conclusion.clone()
    } else {
        localize_report_conclusion(locale, report)
    };
    let summary_title = match locale {
        Locale::ZhCn => "TruthByte 诊断汇总",
        Locale::ZhTw => "TruthByte 診斷摘要",
        Locale::Ja => "TruthByte 診断サマリー",
        Locale::En => "TruthByte Diagnostic Summary",
    };
    let health_label = match locale {
        Locale::ZhCn => "健康评分",
        Locale::ZhTw => "健康評分",
        Locale::Ja => "健康スコア",
        Locale::En => "Health Score",
    };
    let tested_label = match locale {
        Locale::ZhCn => "已测/有效",
        Locale::ZhTw => "已測/有效",
        Locale::Ja => "テスト/有効",
        Locale::En => "Tested/Valid",
    };
    let total_label = match locale {
        Locale::ZhCn => "目标总量",
        Locale::ZhTw => "目標總量",
        Locale::Ja => "総容量",
        Locale::En => "Total Target",
    };
    let status_header = match locale {
        Locale::ZhCn => "状态",
        Locale::ZhTw => "狀態",
        Locale::Ja => "状態",
        Locale::En => "Status",
    };
    let conclusion_header = match locale {
        Locale::ZhCn => "结论",
        Locale::ZhTw => "結論",
        Locale::Ja => "結論",
        Locale::En => "Conclusion",
    };

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
    let locale = detect_locale();
    if args.len() < 2 {
        let intro = match locale {
            Locale::ZhCn => "TruthByte - U盘写入/校验工具。",
            Locale::ZhTw => "TruthByte - USB 寫入/驗證工具。",
            Locale::Ja => "TruthByte - USB書き込み/検証ツール。",
            Locale::En => "TruthByte - USB write/verify tool.",
        };
        let usage = match locale {
            Locale::ZhCn => "用法: {} <文件路径> [大小上限MB] [--force]",
            Locale::ZhTw => "用法: {} <檔案路徑> [大小上限MB] [--force]",
            Locale::Ja => "使い方: {} <ファイルパス> [サイズ上限MB] [--force]",
            Locale::En => "Usage: {} <file_path> [size_limit_mb] [--force]",
        };
        let example = match locale {
            Locale::ZhCn => "示例: {} /Volumes/USB/test.dat 1024 --force",
            Locale::ZhTw => "示例: {} /Volumes/USB/test.dat 1024 --force",
            Locale::Ja => "例: {} /Volumes/USB/test.dat 1024 --force",
            Locale::En => "Example: {} /Volumes/USB/test.dat 1024 --force",
        };
        let note = match locale {
            Locale::ZhCn => "说明: size_limit_mb 可选；0 表示不限制。",
            Locale::ZhTw => "說明: size_limit_mb 可選；0 表示不限制。",
            Locale::Ja => "注: size_limit_mb は任意。0 は無制限。",
            Locale::En => "Note: size_limit_mb is optional; 0 means no limit.",
        };
        println!("{intro}");
        println!("{usage}", args[0]);
        println!("{example}", args[0]);
        println!("{note}");
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
            let message = match locale {
                Locale::ZhCn => "未知参数",
                Locale::ZhTw => "未知參數",
                Locale::Ja => "想定外の引数",
                Locale::En => "Unexpected argument",
            };
            eprintln!("[ERROR] {}: {}", message, arg);
            return 2;
        }
    }

    let limit_mb: u64 = match limit_arg {
        Some(value) => match value.parse() {
            Ok(parsed) => parsed,
            Err(_) => {
                let message = match locale {
                    Locale::ZhCn => "size_limit_mb 无效",
                    Locale::ZhTw => "size_limit_mb 無效",
                    Locale::Ja => "size_limit_mb が無効です",
                    Locale::En => "Invalid size_limit_mb",
                };
                eprintln!("[ERROR] {}: {}", message, value);
                return 2;
            }
        },
        None => 0,
    };

    let path = Path::new(file_path);
    if path.is_dir() {
        let message = match locale {
            Locale::ZhCn => "目标路径是目录",
            Locale::ZhTw => "目標路徑是目錄",
            Locale::Ja => "対象パスはディレクトリです",
            Locale::En => "Target path is a directory",
        };
        eprintln!("[ERROR] {}: {}", message, file_path);
        return 2;
    }

    if path.exists() && !force {
        let prompt = match locale {
            Locale::ZhCn => "[WARN] {} 已存在，将被覆盖。继续? [y/N]: ",
            Locale::ZhTw => "[WARN] {} 已存在，將被覆寫。繼續? [y/N]: ",
            Locale::Ja => "[WARN] {} は既に存在し、上書きされます。続行しますか? [y/N]: ",
            Locale::En => "[WARN] {} exists and will be overwritten. Continue? [y/N]: ",
        };
        print!(prompt, file_path);
        if io::stdout().flush().is_err() {
            let message = match locale {
                Locale::ZhCn => "提示确认失败。",
                Locale::ZhTw => "提示確認失敗。",
                Locale::Ja => "確認のプロンプトに失敗しました。",
                Locale::En => "Failed to prompt for confirmation.",
            };
            eprintln!("[ERROR] {}", message);
            return 2;
        }
        let mut response = String::new();
        if io::stdin().read_line(&mut response).is_err() {
            let message = match locale {
                Locale::ZhCn => "读取确认失败。",
                Locale::ZhTw => "讀取確認失敗。",
                Locale::Ja => "確認の読み取りに失敗しました。",
                Locale::En => "Failed to read confirmation.",
            };
            eprintln!("[ERROR] {}", message);
            return 2;
        }
        let response = response.trim().to_ascii_lowercase();
        if response != "y" && response != "yes" {
            let message = match locale {
                Locale::ZhCn => "用户已取消。",
                Locale::ZhTw => "使用者已取消。",
                Locale::Ja => "ユーザーが中止しました。",
                Locale::En => "Aborted by user.",
            };
            println!("[INFO] {}", message);
            return 1;
        }
    }

    match run_write_verify(file_path, limit_mb, AppConfig::default()) {
        Ok(outcome) => {
            if outcome.bytes_written == 0 {
                let message = match locale {
                    Locale::ZhCn => "未写入数据；已跳过校验阶段。",
                    Locale::ZhTw => "未寫入資料；已跳過驗證階段。",
                    Locale::Ja => "データが書き込まれていないため、検証フェーズをスキップしました。",
                    Locale::En => "No data written; verify phase skipped.",
                };
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
            let message = match locale {
                Locale::ZhCn => "写入阶段失败",
                Locale::ZhTw => "寫入階段失敗",
                Locale::Ja => "書き込みフェーズに失敗しました",
                Locale::En => "Write phase failed",
            };
            println!("[ERROR] {}: {}", message, e);
            2
        }
        Err(RunError::Verify(e)) => {
            let message = match locale {
                Locale::ZhCn => "校验阶段失败",
                Locale::ZhTw => "驗證階段失敗",
                Locale::Ja => "検証フェーズに失敗しました",
                Locale::En => "Verify phase failed",
            };
            println!("[ERROR] {}: {}", message, e);
            2
        }
    }
}
