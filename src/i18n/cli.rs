use std::env;

use crate::core_logic::{DiagnosisReport, DriveHealthStatus};

#[derive(Copy, Clone)]
pub enum Locale {
    En,
    ZhCn,
    ZhTw,
    Ja,
}

fn parse_locale(value: &str) -> Option<Locale> {
    let locale = value.to_lowercase();
    if locale.starts_with("zh-cn") || locale.starts_with("zh_cn") || locale.starts_with("zh-hans") {
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

pub fn detect_locale() -> Locale {
    for key in ["LC_ALL", "LC_MESSAGES", "LANG"] {
        if let Ok(value) = env::var(key) {
            if let Some(locale) = parse_locale(&value) {
                return locale;
            }
        }
    }
    Locale::En
}

pub fn status_label(locale: Locale, status: DriveHealthStatus) -> &'static str {
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

pub fn localize_report_conclusion(locale: Locale, report: &DiagnosisReport) -> String {
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

pub fn summary_title(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "TruthByte 诊断汇总",
        Locale::ZhTw => "TruthByte 診斷摘要",
        Locale::Ja => "TruthByte 診断サマリー",
        Locale::En => "TruthByte Diagnostic Summary",
    }
}

pub fn health_label(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "健康评分",
        Locale::ZhTw => "健康評分",
        Locale::Ja => "健康スコア",
        Locale::En => "Health Score",
    }
}

pub fn tested_label(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "已测/有效",
        Locale::ZhTw => "已測/有效",
        Locale::Ja => "テスト/有効",
        Locale::En => "Tested/Valid",
    }
}

pub fn total_label(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "目标总量",
        Locale::ZhTw => "目標總量",
        Locale::Ja => "総容量",
        Locale::En => "Total Target",
    }
}

pub fn status_header(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "状态",
        Locale::ZhTw => "狀態",
        Locale::Ja => "状態",
        Locale::En => "Status",
    }
}

pub fn conclusion_header(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "结论",
        Locale::ZhTw => "結論",
        Locale::Ja => "結論",
        Locale::En => "Conclusion",
    }
}

pub fn cli_intro(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "TruthByte - U盘写入/校验工具。",
        Locale::ZhTw => "TruthByte - USB 寫入/驗證工具。",
        Locale::Ja => "TruthByte - USB書き込み/検証ツール。",
        Locale::En => "TruthByte - USB write/verify tool.",
    }
}

pub fn cli_usage(locale: Locale, binary: &str) -> String {
    let template = match locale {
        Locale::ZhCn => "用法: {} <文件路径> [大小上限MB] [--force]",
        Locale::ZhTw => "用法: {} <檔案路徑> [大小上限MB] [--force]",
        Locale::Ja => "使い方: {} <ファイルパス> [サイズ上限MB] [--force]",
        Locale::En => "Usage: {} <file_path> [size_limit_mb] [--force]",
    };
    template.replace("{}", binary)
}

pub fn cli_example(locale: Locale, binary: &str) -> String {
    let template = match locale {
        Locale::ZhCn => "示例: {} /Volumes/USB/test.dat 1024 --force",
        Locale::ZhTw => "示例: {} /Volumes/USB/test.dat 1024 --force",
        Locale::Ja => "例: {} /Volumes/USB/test.dat 1024 --force",
        Locale::En => "Example: {} /Volumes/USB/test.dat 1024 --force",
    };
    template.replace("{}", binary)
}

pub fn cli_note(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "说明: size_limit_mb 可选；0 表示不限制。",
        Locale::ZhTw => "說明: size_limit_mb 可選；0 表示不限制。",
        Locale::Ja => "注: size_limit_mb は任意。0 は無制限。",
        Locale::En => "Note: size_limit_mb is optional; 0 means no limit.",
    }
}

pub fn cli_unexpected_arg(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "未知参数",
        Locale::ZhTw => "未知參數",
        Locale::Ja => "想定外の引数",
        Locale::En => "Unexpected argument",
    }
}

pub fn cli_invalid_limit(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "size_limit_mb 无效",
        Locale::ZhTw => "size_limit_mb 無效",
        Locale::Ja => "size_limit_mb が無効です",
        Locale::En => "Invalid size_limit_mb",
    }
}

pub fn cli_target_is_dir(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "目标路径是目录",
        Locale::ZhTw => "目標路徑是目錄",
        Locale::Ja => "対象パスはディレクトリです",
        Locale::En => "Target path is a directory",
    }
}

pub fn cli_overwrite_prompt(locale: Locale, file_path: &str) -> String {
    let template = match locale {
        Locale::ZhCn => "[WARN] {} 已存在，将被覆盖。继续? [y/N]: ",
        Locale::ZhTw => "[WARN] {} 已存在，將被覆寫。繼續? [y/N]: ",
        Locale::Ja => "[WARN] {} は既に存在し、上書きされます。続行しますか? [y/N]: ",
        Locale::En => "[WARN] {} exists and will be overwritten. Continue? [y/N]: ",
    };
    template.replace("{}", file_path)
}

pub fn cli_prompt_flush_failed(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "提示确认失败。",
        Locale::ZhTw => "提示確認失敗。",
        Locale::Ja => "確認のプロンプトに失敗しました。",
        Locale::En => "Failed to prompt for confirmation.",
    }
}

pub fn cli_prompt_read_failed(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "读取确认失败。",
        Locale::ZhTw => "讀取確認失敗。",
        Locale::Ja => "確認の読み取りに失敗しました。",
        Locale::En => "Failed to read confirmation.",
    }
}

pub fn cli_user_aborted(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "用户已取消。",
        Locale::ZhTw => "使用者已取消。",
        Locale::Ja => "ユーザーが中止しました。",
        Locale::En => "Aborted by user.",
    }
}

pub fn cli_no_data_written(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "未写入数据；已跳过校验阶段。",
        Locale::ZhTw => "未寫入資料；已跳過驗證階段。",
        Locale::Ja => "データが書き込まれていないため、検証フェーズをスキップしました。",
        Locale::En => "No data written; verify phase skipped.",
    }
}

pub fn cli_write_phase_failed(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "写入阶段失败",
        Locale::ZhTw => "寫入階段失敗",
        Locale::Ja => "書き込みフェーズに失敗しました",
        Locale::En => "Write phase failed",
    }
}

pub fn cli_verify_phase_failed(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "校验阶段失败",
        Locale::ZhTw => "驗證階段失敗",
        Locale::Ja => "検証フェーズに失敗しました",
        Locale::En => "Verify phase failed",
    }
}
