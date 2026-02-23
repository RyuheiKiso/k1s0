use std::fmt::Write as FmtWrite;

use anyhow::Result;

use super::helpers::{
    prompt_db_selection, prompt_name_or_select, scan_existing_databases, scan_existing_dirs,
};
use super::types::{
    ApiStyle, DetailConfig, Framework, GenerateConfig, Kind, LangFw, Language, Tier,
    ALL_API_STYLES, ALL_KINDS, ALL_RDBMS, API_LABELS, KIND_LABELS, RDBMS_LABELS,
};
use crate::prompt;

// ============================================================================
// 各ステップ
// ============================================================================

pub(super) enum StepResult<T> {
    Value(T),
    Skip,
    Back,
}

/// ステップ1: 種別選択
pub(super) fn step_kind() -> Result<Option<Kind>> {
    let idx = prompt::select_prompt("何を生成しますか？", KIND_LABELS)?;
    Ok(idx.map(|i| ALL_KINDS[i]))
}

/// ステップ2: Tier選択
pub(super) fn step_tier(kind: Kind) -> Result<Option<Tier>> {
    let available = kind.available_tiers();
    let labels: Vec<&str> = available
        .iter()
        .map(k1s0_core::commands::generate::Tier::label)
        .collect();
    let idx = prompt::select_prompt("Tier を選択してください", &labels)?;
    Ok(idx.map(|i| available[i]))
}

/// ステップ3: 配置先指定
///
/// `Tier::System` の場合は配置先不要のためスキップ (`StepResult::Skip`)。
/// Esc が押された場合は `StepResult::Back` を返す。
pub(super) fn step_placement(tier: Tier) -> Result<StepResult<Option<String>>> {
    match tier {
        Tier::System => Ok(StepResult::Skip),
        Tier::Business => {
            let existing = scan_existing_dirs("regions/business");
            let name = prompt_name_or_select(
                "領域名を入力または選択してください",
                "領域名を入力してください",
                &existing,
            )?;
            match name {
                Some(n) => Ok(StepResult::Value(Some(n))),
                None => Ok(StepResult::Back),
            }
        }
        Tier::Service => {
            let existing = scan_existing_dirs("regions/service");
            let name = prompt_name_or_select(
                "サービス名を入力または選択してください",
                "サービス名を入力してください",
                &existing,
            )?;
            match name {
                Some(n) => Ok(StepResult::Value(Some(n))),
                None => Ok(StepResult::Back),
            }
        }
    }
}

/// ステップ4: 言語/FW選択
pub(super) fn step_lang_fw(kind: Kind) -> Result<Option<LangFw>> {
    match kind {
        Kind::Server => {
            let items = &["Go", "Rust"];
            let idx = prompt::select_prompt("言語を選択してください", items)?;
            Ok(idx.map(|i| {
                LangFw::Language(match i {
                    0 => Language::Go,
                    1 => Language::Rust,
                    _ => unreachable!(),
                })
            }))
        }
        Kind::Client => {
            let items = &["React", "Flutter"];
            let idx = prompt::select_prompt("フレームワークを選択してください", items)?;
            Ok(idx.map(|i| {
                LangFw::Framework(match i {
                    0 => Framework::React,
                    1 => Framework::Flutter,
                    _ => unreachable!(),
                })
            }))
        }
        Kind::Library => {
            let items = &["Go", "Rust", "TypeScript", "Dart", "Python", "Swift"];
            let idx = prompt::select_prompt("言語を選択してください", items)?;
            Ok(idx.map(|i| {
                LangFw::Language(match i {
                    0 => Language::Go,
                    1 => Language::Rust,
                    2 => Language::TypeScript,
                    3 => Language::Dart,
                    4 => Language::Python,
                    5 => Language::Swift,
                    _ => unreachable!(),
                })
            }))
        }
        Kind::Database => {
            let db_name = prompt::input_prompt("データベース名を入力してください");
            match db_name {
                Ok(name) => {
                    let idx = prompt::select_prompt("RDBMS を選択してください", RDBMS_LABELS)?;
                    match idx {
                        Some(i) => Ok(Some(LangFw::Database {
                            name,
                            rdbms: ALL_RDBMS[i],
                        })),
                        None => Ok(None),
                    }
                }
                Err(_) => Ok(None),
            }
        }
    }
}

/// ステップ5: 詳細設定
pub(super) fn step_detail(
    kind: Kind,
    tier: Tier,
    placement: Option<&str>,
    _lang_fw: &LangFw,
) -> Result<Option<DetailConfig>> {
    match kind {
        Kind::Server => step_detail_server(tier, placement),
        Kind::Client => Ok(step_detail_client(tier, placement)),
        Kind::Library => Ok(step_detail_library()),
        Kind::Database => Ok(Some(DetailConfig::default())),
    }
}

/// サーバー詳細設定
fn step_detail_server(tier: Tier, placement: Option<&str>) -> Result<Option<DetailConfig>> {
    // サービス名: service Tier ではステップ3 のサービス名を使う
    let service_name = if tier == Tier::Service {
        placement.map(str::to_owned)
    } else {
        match prompt::input_prompt("サービス名を入力してください") {
            Ok(n) => Some(n),
            Err(_) => return Ok(None),
        }
    };

    // API方式
    let api_indices =
        prompt::multi_select_prompt("API 方式を選択してください（複数選択可）", API_LABELS)?;
    let api_styles: Vec<ApiStyle> = match api_indices {
        Some(indices) => indices.iter().map(|&i| ALL_API_STYLES[i]).collect(),
        None => return Ok(None),
    };

    // DB追加
    let add_db = prompt::yes_no_prompt("データベースを追加しますか？")?;
    let db = match add_db {
        Some(true) => {
            // 既存DBの探索
            let existing_dbs = scan_existing_databases();

            prompt_db_selection(&existing_dbs)?
        }
        Some(false) => None,
        None => return Ok(None),
    };

    // Kafka
    let Some(kafka) = prompt::yes_no_prompt("メッセージング (Kafka) を有効にしますか？")?
    else {
        return Ok(None);
    };

    // Redis
    let Some(redis) = prompt::yes_no_prompt("キャッシュ (Redis) を有効にしますか？")?
    else {
        return Ok(None);
    };

    // BFF (service Tier + GraphQL 時のみ)
    let bff_language = if tier == Tier::Service && api_styles.contains(&ApiStyle::GraphQL) {
        let want_bff = prompt::yes_no_prompt("GraphQL BFF を生成しますか？")?;
        match want_bff {
            Some(true) => {
                let items = &["Go", "Rust"];
                let idx = prompt::select_prompt("BFF の言語を選択してください", items)?;
                match idx {
                    Some(0) => Some(Language::Go),
                    Some(1) => Some(Language::Rust),
                    Some(_) => unreachable!(),
                    None => return Ok(None),
                }
            }
            Some(false) => None,
            None => return Ok(None),
        }
    } else {
        None
    };

    Ok(Some(DetailConfig {
        name: service_name,
        api_styles,
        db,
        kafka,
        redis,
        bff_language,
    }))
}

/// クライアント詳細設定
fn step_detail_client(tier: Tier, placement: Option<&str>) -> Option<DetailConfig> {
    let app_name = if tier == Tier::Service {
        // service Tier: ステップ3のサービス名をアプリ名として使用
        placement.map(str::to_owned)
    } else {
        // business Tier: アプリ名入力
        match prompt::input_prompt("アプリ名を入力してください") {
            Ok(n) => Some(n),
            Err(_) => return None,
        }
    };

    Some(DetailConfig {
        name: app_name,
        ..DetailConfig::default()
    })
}

/// ライブラリ詳細設定
fn step_detail_library() -> Option<DetailConfig> {
    let Ok(lib_name) = prompt::input_prompt("ライブラリ名を入力してください") else {
        return None;
    };

    Some(DetailConfig {
        name: Some(lib_name),
        ..DetailConfig::default()
    })
}

// ============================================================================
// 確認表示
// ============================================================================

pub(super) fn print_confirmation(config: &GenerateConfig) {
    print!("{}", format_confirmation(config));
}

/// 確認画面の内容を文字列として構築する（テスト可能）。
pub(super) fn format_confirmation(config: &GenerateConfig) -> String {
    let mut out = String::new();
    out.push_str("\n[確認] 以下の内容で生成します。よろしいですか？\n");
    writeln!(out, "    種別:     {}", config.kind.label()).unwrap();
    writeln!(out, "    Tier:     {}", config.tier.as_str()).unwrap();

    // 配置先
    if let Some(ref p) = config.placement {
        match config.tier {
            Tier::Business => writeln!(out, "    領域:     {p}").unwrap(),
            Tier::Service => writeln!(out, "    サービス: {p}").unwrap(),
            Tier::System => {}
        }
    }

    match config.kind {
        Kind::Server => {
            // service Tier では placement で既にサービス名を表示済みのため、
            // detail.name の表示をスキップする
            if config.tier != Tier::Service {
                if let Some(ref name) = config.detail.name {
                    writeln!(out, "    サービス: {name}").unwrap();
                }
            }
            if let LangFw::Language(lang) = config.lang_fw {
                writeln!(out, "    言語:     {}", lang.as_str()).unwrap();
            }
            if !config.detail.api_styles.is_empty() {
                let api_strs: Vec<&str> = config
                    .detail
                    .api_styles
                    .iter()
                    .map(k1s0_core::commands::generate::ApiStyle::short_label)
                    .collect();
                writeln!(out, "    API:      {}", api_strs.join(", ")).unwrap();
            }
            // BFF 情報（service Tier + GraphQL 時のみ表示）
            if let Some(bff_lang) = config.detail.bff_language {
                writeln!(out, "    BFF:      あり ({})", bff_lang.as_str()).unwrap();
            }
            match &config.detail.db {
                Some(db) => writeln!(out, "    DB:       {db}").unwrap(),
                None => out.push_str("    DB:       なし\n"),
            }
            writeln!(
                out,
                "    Kafka:    {}",
                if config.detail.kafka {
                    "有効"
                } else {
                    "無効"
                }
            )
            .unwrap();
            writeln!(
                out,
                "    Redis:    {}",
                if config.detail.redis {
                    "有効"
                } else {
                    "無効"
                }
            )
            .unwrap();
        }
        Kind::Client => {
            if let LangFw::Framework(fw) = config.lang_fw {
                writeln!(out, "    フレームワーク: {}", fw.as_str()).unwrap();
            }
            if let Some(ref name) = config.detail.name {
                writeln!(out, "    アプリ名:       {name}").unwrap();
            }
        }
        Kind::Library => {
            if let LangFw::Language(lang) = config.lang_fw {
                writeln!(out, "    言語:         {}", lang.as_str()).unwrap();
            }
            if let Some(ref name) = config.detail.name {
                writeln!(out, "    ライブラリ名: {name}").unwrap();
            }
        }
        Kind::Database => {
            if let LangFw::Database { ref name, rdbms } = config.lang_fw {
                writeln!(out, "    データベース名: {name}").unwrap();
                writeln!(out, "    RDBMS:          {}", rdbms.as_str()).unwrap();
            }
        }
    }

    out
}

// ============================================================================
// テスト
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_confirmation_with_bff() {
        // BFF 設定が確認画面に表示されること
        let config = GenerateConfig {
            kind: Kind::Server,
            tier: Tier::Service,
            placement: Some("order".to_string()),
            lang_fw: LangFw::Language(Language::Go),
            detail: DetailConfig {
                name: Some("order".to_string()),
                api_styles: vec![ApiStyle::GraphQL],
                db: None,
                kafka: false,
                redis: false,
                bff_language: Some(Language::Go),
            },
        };
        let output = format_confirmation(&config);
        assert!(
            output.contains("BFF:      あり (Go)"),
            "BFF 情報が確認画面に表示されるべき: {output}"
        );
    }

    #[test]
    fn test_print_confirmation_without_bff() {
        // bff_language が None の場合は BFF 行が表示されない
        let config = GenerateConfig {
            kind: Kind::Server,
            tier: Tier::Service,
            placement: Some("order".to_string()),
            lang_fw: LangFw::Language(Language::Go),
            detail: DetailConfig {
                name: Some("order".to_string()),
                api_styles: vec![ApiStyle::GraphQL],
                db: None,
                kafka: false,
                redis: false,
                bff_language: None,
            },
        };
        let output = format_confirmation(&config);
        assert!(
            !output.contains("BFF:"),
            "bff_language=None の場合は BFF 行は非表示: {output}"
        );
    }

    #[test]
    fn test_print_confirmation_bff_rust() {
        // BFF 言語が Rust の場合
        let config = GenerateConfig {
            kind: Kind::Server,
            tier: Tier::Service,
            placement: Some("order".to_string()),
            lang_fw: LangFw::Language(Language::Go),
            detail: DetailConfig {
                name: Some("order".to_string()),
                api_styles: vec![ApiStyle::GraphQL],
                db: None,
                kafka: false,
                redis: false,
                bff_language: Some(Language::Rust),
            },
        };
        let output = format_confirmation(&config);
        assert!(
            output.contains("BFF:      あり (Rust)"),
            "BFF (Rust) 情報が確認画面に表示されるべき: {output}"
        );
    }

    #[test]
    fn test_print_confirmation_server_basic() {
        // サーバーの基本確認表示がそのまま動作すること（既存の互換性）
        let config = GenerateConfig {
            kind: Kind::Server,
            tier: Tier::System,
            placement: None,
            lang_fw: LangFw::Language(Language::Go),
            detail: DetailConfig {
                name: Some("auth".to_string()),
                api_styles: vec![ApiStyle::Rest],
                db: None,
                kafka: false,
                redis: false,
                bff_language: None,
            },
        };
        let output = format_confirmation(&config);
        assert!(output.contains("種別:     サーバー"));
        assert!(output.contains("Tier:     system"));
        assert!(output.contains("サービス: auth"));
        assert!(output.contains("言語:     Go"));
        assert!(output.contains("API:      REST"));
        assert!(output.contains("DB:       なし"));
        assert!(output.contains("Kafka:    無効"));
        assert!(output.contains("Redis:    無効"));
        assert!(!output.contains("BFF:"));
    }
}
