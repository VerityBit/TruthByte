use truthbyte::{DiagnosisReport, DriveHealthStatus};

#[derive(Clone, Copy)]
pub enum Locale {
    En,
    ZhCn,
    ZhTw,
    Ja,
}

impl Locale {
    pub fn from_tag(tag: &str) -> Self {
        match tag {
            "zh-CN" => Self::ZhCn,
            "zh-TW" => Self::ZhTw,
            "ja" => Self::Ja,
            _ => Self::En,
        }
    }
}

pub fn localize_error(locale: Locale, message: &str) -> String {
    let translate_simple = |en: &str, zh_cn: &str, zh_tw: &str, ja: &str| match locale {
        Locale::En => en,
        Locale::ZhCn => zh_cn,
        Locale::ZhTw => zh_tw,
        Locale::Ja => ja,
    };

    let localize_detail = |detail: &str| match detail {
        "Parent directory does not exist." => translate_simple(
            "Parent directory does not exist.",
            "上级目录不存在。",
            "上層目錄不存在。",
            "親ディレクトリが存在しません。",
        )
        .to_string(),
        _ => detail.to_string(),
    };

    match message {
        "Diagnosis is already running." => {
            return translate_simple(
                "Diagnosis is already running.",
                "诊断已在运行中。",
                "診斷正在執行中。",
                "診断はすでに実行中です。",
            )
            .to_string();
        }
        "No active diagnosis to stop." => {
            return translate_simple(
                "No active diagnosis to stop.",
                "没有正在运行的诊断可停止。",
                "沒有正在執行的診斷可停止。",
                "停止できる実行中の診断がありません。",
            )
            .to_string();
        }
        "Parent directory does not exist." => {
            return translate_simple(
                "Parent directory does not exist.",
                "上级目录不存在。",
                "上層目錄不存在。",
                "親ディレクトリが存在しません。",
            )
            .to_string();
        }
        _ => {}
    }

    if let Some(detail) = message.strip_prefix("Verify phase failed: ") {
        return format!(
            "{}: {}",
            translate_simple(
                "Verify phase failed",
                "校验阶段失败",
                "驗證階段失敗",
                "検証フェーズに失敗しました"
            ),
            localize_detail(detail)
        );
    }

    if let Some(detail) = message.strip_prefix("Write phase failed: ") {
        return format!(
            "{}: {}",
            translate_simple(
                "Write phase failed",
                "写入阶段失败",
                "寫入階段失敗",
                "書き込みフェーズに失敗しました"
            ),
            localize_detail(detail)
        );
    }

    if let Some(detail) = message.strip_prefix("Unable to open target for writing: ") {
        return format!(
            "{}: {}",
            translate_simple(
                "Unable to open target for writing",
                "无法打开目标进行写入",
                "無法開啟目標進行寫入",
                "書き込み用に対象を開けませんでした"
            ),
            localize_detail(detail)
        );
    }

    if let Some(detail) = message.strip_prefix("Unable to open target for reading: ") {
        return format!(
            "{}: {}",
            translate_simple(
                "Unable to open target for reading",
                "无法打开目标进行读取",
                "無法開啟目標進行讀取",
                "読み取り用に対象を開けませんでした"
            ),
            localize_detail(detail)
        );
    }

    if let Some(detail) = message.strip_prefix("Failed to sync data: ") {
        return format!(
            "{}: {}",
            translate_simple(
                "Failed to sync data",
                "同步数据失败",
                "同步資料失敗",
                "データの同期に失敗しました"
            ),
            localize_detail(detail)
        );
    }

    if let Some(rest) = message.strip_prefix("Write failure at offset ") {
        if let Some((offset, detail)) = rest.split_once(": ") {
            return format!(
                "{} {}: {}",
                translate_simple(
                    "Write failure at offset",
                    "写入失败，偏移",
                    "寫入失敗，偏移",
                    "オフセットで書き込みに失敗しました"
                ),
                offset,
                localize_detail(detail)
            );
        }
    }

    if let Some(rest) = message.strip_prefix("Unable to seek past read failure at offset ") {
        if let Some((offset, detail)) = rest.split_once(": ") {
            return format!(
                "{} {}: {}",
                translate_simple(
                    "Unable to seek past read failure at offset",
                    "无法越过读取失败位置，偏移",
                    "無法越過讀取失敗位置，偏移",
                    "読み取り失敗位置を超えてシークできませんでした"
                ),
                offset,
                localize_detail(detail)
            );
        }
    }

    message.to_string()
}

pub fn localize_conclusion(locale: Locale, message: &str) -> String {
    match message {
        "No data written; verification skipped." => match locale {
            Locale::En => "No data written; verification skipped.".to_string(),
            Locale::ZhCn => "未写入数据；已跳过校验。".to_string(),
            Locale::ZhTw => "未寫入資料；已跳過驗證。".to_string(),
            Locale::Ja => "データが書き込まれていないため、検証をスキップしました。"
                .to_string(),
        },
        _ => message.to_string(),
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
