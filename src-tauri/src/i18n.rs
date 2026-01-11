use truthbyte::{DiagnosisReport, DriveHealthStatus};

#[derive(Clone, Copy)]
pub enum Locale {
    En,
    Es,
    Fr,
    De,
    Ru,
    Ko,
    ZhCn,
    ZhTw,
    Ja,
}

impl Locale {
    pub fn from_tag(tag: &str) -> Self {
        let normalized = tag.trim().to_lowercase();

        if normalized.starts_with("zh-cn") || normalized.starts_with("zh-hans") {
            return Self::ZhCn;
        }
        if normalized.starts_with("zh-tw")
            || normalized.starts_with("zh-hant")
            || normalized.starts_with("zh-hk")
        {
            return Self::ZhTw;
        }
        if normalized.starts_with("ja") {
            return Self::Ja;
        }
        if normalized.starts_with("ko") {
            return Self::Ko;
        }
        if normalized.starts_with("es") {
            return Self::Es;
        }
        if normalized.starts_with("fr") {
            return Self::Fr;
        }
        if normalized.starts_with("de") {
            return Self::De;
        }
        if normalized.starts_with("ru") {
            return Self::Ru;
        }

        Self::En
    }
}

pub fn localize_error(locale: Locale, message: &str) -> String {
    let translate_simple = |en: &str,
                            es: &str,
                            fr: &str,
                            de: &str,
                            ru: &str,
                            ko: &str,
                            zh_cn: &str,
                            zh_tw: &str,
                            ja: &str| match locale {
        Locale::En => en,
        Locale::Es => es,
        Locale::Fr => fr,
        Locale::De => de,
        Locale::Ru => ru,
        Locale::Ko => ko,
        Locale::ZhCn => zh_cn,
        Locale::ZhTw => zh_tw,
        Locale::Ja => ja,
    };

    let localize_detail = |detail: &str| match detail {
        "Parent directory does not exist." => translate_simple(
            "Parent directory does not exist.",
            "El directorio padre no existe.",
            "Le répertoire parent n'existe pas.",
            "Das übergeordnete Verzeichnis existiert nicht.",
            "Родительский каталог не существует.",
            "상위 디렉터리가 존재하지 않습니다.",
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
                "El diagnóstico ya se está ejecutando.",
                "Le diagnostic est déjà en cours.",
                "Die Diagnose läuft bereits.",
                "Диагностика уже выполняется.",
                "진단이 이미 실행 중입니다.",
                "诊断已在运行中。",
                "診斷已在執行中。",
                "診断は既に実行中です。",
            )
            .to_string();
        }
        "No active diagnosis to stop." => {
            return translate_simple(
                "No active diagnosis to stop.",
                "No hay un diagnóstico activo para detener.",
                "Aucun diagnostic actif à arrêter.",
                "Es gibt keine aktive Diagnose zum Stoppen.",
                "Нет активной диагностики для остановки.",
                "중지할 활성 진단이 없습니다.",
                "没有正在运行的诊断可停止。",
                "沒有正在執行的診斷可停止。",
                "停止できる診断は実行中ではありません。",
            )
            .to_string();
        }
        "Parent directory does not exist." => {
            return translate_simple(
                "Parent directory does not exist.",
                "El directorio padre no existe.",
                "Le répertoire parent n'existe pas.",
                "Das übergeordnete Verzeichnis existiert nicht.",
                "Родительский каталог не существует.",
                "상위 디렉터리가 존재하지 않습니다.",
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
                "La fase de verificación falló",
                "La phase de vérification a échoué",
                "Die Verifizierungsphase ist fehlgeschlagen",
                "Сбой на этапе проверки",
                "검증 단계 실패",
                "校验阶段失败",
                "驗證階段失敗",
                "検証フェーズに失敗しました",
            ),
            localize_detail(detail)
        );
    }

    if let Some(detail) = message.strip_prefix("Write phase failed: ") {
        return format!(
            "{}: {}",
            translate_simple(
                "Write phase failed",
                "La fase de escritura falló",
                "La phase d'écriture a échoué",
                "Die Schreibphase ist fehlgeschlagen",
                "Сбой на этапе записи",
                "쓰기 단계 실패",
                "写入阶段失败",
                "寫入階段失敗",
                "書き込みフェーズに失敗しました",
            ),
            localize_detail(detail)
        );
    }

    if let Some(detail) = message.strip_prefix("Unable to open target for writing: ") {
        return format!(
            "{}: {}",
            translate_simple(
                "Unable to open target for writing",
                "No se pudo abrir el destino para escritura",
                "Impossible d'ouvrir la cible en écriture",
                "Ziel konnte nicht zum Schreiben geöffnet werden",
                "Не удалось открыть цель для записи",
                "쓰기 위해 대상을 열 수 없습니다",
                "无法打开目标进行写入",
                "無法開啟目標以寫入",
                "書き込み用に対象を開けません",
            ),
            localize_detail(detail)
        );
    }

    if let Some(detail) = message.strip_prefix("Unable to open target for reading: ") {
        return format!(
            "{}: {}",
            translate_simple(
                "Unable to open target for reading",
                "No se pudo abrir el destino para lectura",
                "Impossible d'ouvrir la cible en lecture",
                "Ziel konnte nicht zum Lesen geöffnet werden",
                "Не удалось открыть цель для чтения",
                "읽기 위해 대상을 열 수 없습니다",
                "无法打开目标进行读取",
                "無法開啟目標以讀取",
                "読み取り用に対象を開けません",
            ),
            localize_detail(detail)
        );
    }

    if let Some(detail) = message.strip_prefix("Failed to sync data: ") {
        return format!(
            "{}: {}",
            translate_simple(
                "Failed to sync data",
                "No se pudieron sincronizar los datos",
                "Échec de la synchronisation des données",
                "Daten konnten nicht synchronisiert werden",
                "Не удалось синхронизировать данные",
                "데이터 동기화 실패",
                "同步数据失败",
                "同步資料失敗",
                "データの同期に失敗しました",
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
                    "Fallo de escritura en el desplazamiento",
                    "Échec d'écriture à l'octet",
                    "Schreibfehler bei Offset",
                    "Сбой записи на смещении",
                    "오프셋에서 쓰기 실패",
                    "写入失败，偏移",
                    "寫入失敗，位移",
                    "オフセットで書き込み失敗",
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
                    "No se pudo avanzar después del fallo de lectura en el desplazamiento",
                    "Impossible de dépasser l'échec de lecture à l'octet",
                    "Nach dem Lesefehler bei Offset konnte nicht weitergesucht werden",
                    "Не удалось продолжить после ошибки чтения на смещении",
                    "오프셋에서 읽기 실패 이후로 이동할 수 없습니다",
                    "无法越过读取失败位置，偏移",
                    "無法越過讀取失敗位置，位移",
                    "読み取り失敗位置を超えてシークできません",
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
            Locale::Es => "No se escribieron datos; se omitió la verificación.".to_string(),
            Locale::Fr => "Aucune donnée écrite ; vérification ignorée.".to_string(),
            Locale::De => "Keine Daten geschrieben; Verifizierung übersprungen.".to_string(),
            Locale::Ru => "Данные не были записаны; проверка пропущена.".to_string(),
            Locale::Ko => "데이터가 기록되지 않아 검증을 건너뛰었습니다.".to_string(),
            Locale::ZhCn => "未写入数据；已跳过校验。".to_string(),
            Locale::ZhTw => "未寫入資料；已跳過驗證。".to_string(),
            Locale::Ja => "データが書き込まれていないため、検証をスキップしました。".to_string(),
        },
        _ => message.to_string(),
    }
}

pub fn localize_report_conclusion(locale: Locale, report: &DiagnosisReport) -> String {
    let base = match report.status {
        DriveHealthStatus::Healthy => match locale {
            Locale::En => "No inconsistencies detected.",
            Locale::Es => "No se detectaron inconsistencias.",
            Locale::Fr => "Aucune incohérence détectée.",
            Locale::De => "Keine Inkonsistenzen festgestellt.",
            Locale::Ru => "Несоответствий не обнаружено.",
            Locale::Ko => "불일치가 감지되지 않았습니다.",
            Locale::ZhCn => "未发现不一致。",
            Locale::ZhTw => "未發現不一致。",
            Locale::Ja => "不整合は検出されませんでした。",
        },
        DriveHealthStatus::FakeCapacity => match locale {
            Locale::En => "Warning: detected zero-filled data; possible fake capacity.",
            Locale::Es => "Advertencia: se detectaron datos llenos de ceros; posible capacidad falsa.",
            Locale::Fr => "Avertissement : des données remplies de zéros ont été détectées ; capacité potentiellement factice.",
            Locale::De => "Warnung: mit Nullen gefüllte Daten erkannt; mögliche Falschkapazität.",
            Locale::Ru => "Предупреждение: обнаружены заполненные нулями данные; возможна ложная емкость.",
            Locale::Ko => "경고: 0으로 채워진 데이터가 감지되었습니다. 가짜 용량일 수 있습니다.",
            Locale::ZhCn => "警告：检测到全零数据，可能为虚假容量。",
            Locale::ZhTw => "警告：檢測到全零資料，可能為虛假容量。",
            Locale::Ja => "警告：ゼロ埋めデータを検出しました。偽装容量の可能性があります。",
        },
        DriveHealthStatus::PhysicalCorruption => match locale {
            Locale::En => "Warning: detected random corruption or inconsistent data.",
            Locale::Es => "Advertencia: se detectó corrupción aleatoria o datos inconsistentes.",
            Locale::Fr => "Avertissement : corruption aléatoire ou données incohérentes détectées.",
            Locale::De => "Warnung: zufällige Beschädigung oder inkonsistente Daten erkannt.",
            Locale::Ru => "Предупреждение: обнаружена случайная порча или несогласованные данные.",
            Locale::Ko => "경고: 무작위 손상 또는 불일치 데이터가 감지되었습니다.",
            Locale::ZhCn => "警告：检测到随机损坏或不一致数据。",
            Locale::ZhTw => "警告：檢測到隨機損壞或不一致資料。",
            Locale::Ja => "警告：ランダムな破損または不整合データを検出しました。",
        },
        DriveHealthStatus::DataLoss => match locale {
            Locale::En => "Warning: read errors or missing data detected.",
            Locale::Es => "Advertencia: se detectaron errores de lectura o datos faltantes.",
            Locale::Fr => "Avertissement : erreurs de lecture ou données manquantes détectées.",
            Locale::De => "Warnung: Lesefehler oder fehlende Daten erkannt.",
            Locale::Ru => "Предупреждение: обнаружены ошибки чтения или отсутствующие данные.",
            Locale::Ko => "경고: 읽기 오류 또는 누락된 데이터가 감지되었습니다.",
            Locale::ZhCn => "警告：检测到读取错误或数据缺失。",
            Locale::ZhTw => "警告：檢測到讀取錯誤或資料缺失。",
            Locale::Ja => "警告：読み取りエラーまたはデータ欠落を検出しました。",
        },
    };

    let mut conclusion = match locale {
        Locale::En => format!("{base} Errors: {}.", report.error_count),
        Locale::Es => format!("{base} Errores: {}.", report.error_count),
        Locale::Fr => format!("{base} Erreurs : {}.", report.error_count),
        Locale::De => format!("{base} Fehler: {}.", report.error_count),
        Locale::Ru => format!("{base} Ошибки: {}.", report.error_count),
        Locale::Ko => format!("{base} 오류 수: {}.", report.error_count),
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
            Locale::Es => conclusion.push_str(&format!(
                " La verificación terminó antes de tiempo después de {} / {} bytes.",
                report.tested_bytes, report.total_capacity
            )),
            Locale::Fr => conclusion.push_str(&format!(
                " La vérification s'est terminée prématurément après {} / {} octets.",
                report.tested_bytes, report.total_capacity
            )),
            Locale::De => conclusion.push_str(&format!(
                " Die Verifizierung wurde vorzeitig beendet nach {} / {} Bytes.",
                report.tested_bytes, report.total_capacity
            )),
            Locale::Ru => conclusion.push_str(&format!(
                " Проверка завершилась досрочно после {} / {} байт.",
                report.tested_bytes, report.total_capacity
            )),
            Locale::Ko => conclusion.push_str(&format!(
                " 검증이 조기에 종료되었습니다: {} / {} 바이트.",
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
