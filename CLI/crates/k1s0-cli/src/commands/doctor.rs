//! `k1s0 doctor` コマンド
//!
//! 開発環境の健全性をチェックし、問題を診断する。

use clap::{Args, ValueEnum};
use serde::Serialize;

use crate::doctor::{
    check_all_tools, check_tools_by_category, generate_recommendations, generate_summary,
    has_optional_problems, has_required_problems, CheckStatus, CheckSummary, RecommendAction,
    Recommendation, ToolCategory, ToolCheck,
};
use crate::error::{CliError, Result};
use crate::output::output;

/// チェックカテゴリ
#[derive(Debug, Clone, Copy, ValueEnum, Default)]
pub enum CheckCategory {
    /// 全てチェック
    #[default]
    All,
    /// Rust 関連
    Rust,
    /// Go 関連
    Go,
    /// Node.js 関連
    Node,
    /// Flutter 関連
    Flutter,
    /// Protocol Buffers 関連
    Proto,
}

impl From<CheckCategory> for Option<ToolCategory> {
    fn from(cat: CheckCategory) -> Option<ToolCategory> {
        match cat {
            CheckCategory::All => None,
            CheckCategory::Rust => Some(ToolCategory::Rust),
            CheckCategory::Go => Some(ToolCategory::Go),
            CheckCategory::Node => Some(ToolCategory::Node),
            CheckCategory::Flutter => Some(ToolCategory::Flutter),
            CheckCategory::Proto => Some(ToolCategory::Proto),
        }
    }
}

/// `k1s0 doctor` の引数
#[derive(Args, Debug)]
#[command(after_long_help = r#"例:
  k1s0 doctor
  k1s0 doctor --check rust --verbose

開発環境に必要なツールのインストール状況とバージョンを診断します。
"#)]
pub struct DoctorArgs {
    /// 詳細情報を表示
    #[arg(short, long)]
    pub verbose: bool,

    /// JSON 形式で出力
    #[arg(long)]
    pub json: bool,

    /// チェックするカテゴリ
    #[arg(long, value_enum, default_value = "all")]
    pub check: CheckCategory,

    /// 警告をエラーとして扱う
    #[arg(long)]
    pub strict: bool,
}

/// `k1s0 doctor` を実行する
pub fn execute(args: DoctorArgs) -> Result<()> {
    let out = output();

    // チェック実行
    let checks: Vec<ToolCheck> = match Option::<ToolCategory>::from(args.check) {
        Some(category) => check_tools_by_category(category),
        None => check_all_tools(),
    };

    // 推奨アクション生成
    let recommendations = generate_recommendations(&checks);
    let summary = generate_summary(&checks);

    // 出力
    if args.json || out.is_json_mode() {
        output_json(&checks, &recommendations, &summary);
    } else {
        output_human(&checks, &recommendations, &summary, args.verbose);
    }

    // 終了コード決定
    if has_required_problems(&checks) {
        Err(CliError::validation("必須ツールに問題があります")
            .with_hint("上記の推奨アクションを実行してください"))
    } else if args.strict && has_optional_problems(&checks) {
        Err(CliError::validation("オプションツールに問題があります（--strict モード）")
            .with_hint("上記の推奨アクションを実行してください"))
    } else {
        Ok(())
    }
}

/// 人間向け出力
fn output_human(
    checks: &[ToolCheck],
    recommendations: &[Recommendation],
    summary: &CheckSummary,
    verbose: bool,
) {
    let out = output();

    out.newline();
    out.header("k1s0 環境診断");
    out.newline();

    // k1s0 CLI バージョン
    out.header("k1s0 CLI");
    out.list_item("k1s0", &format!("v{}", crate::version()));
    out.newline();

    // 必須ツール
    out.header("必須ツール");
    for check in checks.iter().filter(|c| c.requirement.required) {
        print_tool_status(check, verbose);
    }
    out.newline();

    // オプションツール
    let optional_checks: Vec<_> = checks.iter().filter(|c| !c.requirement.required).collect();
    if !optional_checks.is_empty() {
        out.header("オプションツール");
        for check in optional_checks {
            print_tool_status(check, verbose);
        }
        out.newline();
    }

    // 推奨アクション
    if !recommendations.is_empty() {
        out.header("推奨アクション");
        for (i, rec) in recommendations.iter().enumerate() {
            let prefix = if rec.required { "必須" } else { "推奨" };
            eprintln!(
                "  {}. [{}] {} をインストール: {}",
                i + 1,
                prefix,
                rec.tool_name,
                rec.url
            );
            if verbose {
                if let Some(cmd) = rec.install_command {
                    eprintln!("     コマンド: {}", cmd);
                }
            }
        }
        out.newline();
    }

    // サマリー
    if summary.required_failed == 0 && summary.optional_missing == 0 {
        out.success("全てのツールが正常にインストールされています");
    } else if summary.required_failed == 0 {
        out.success(&format!(
            "必須ツール: {}/{} OK",
            summary.required_ok,
            summary.required_ok + summary.required_failed
        ));
        out.warning(&format!(
            "オプションツール: {}/{} 未インストール",
            summary.optional_missing,
            summary.optional_ok + summary.optional_missing
        ));
    } else {
        out.warning(&format!(
            "必須ツール: {}/{} に問題があります",
            summary.required_failed,
            summary.required_ok + summary.required_failed
        ));
    }
}

/// ツールのステータスを出力
fn print_tool_status(check: &ToolCheck, verbose: bool) {
    let out = output();
    let name = check.requirement.name;

    match &check.status {
        CheckStatus::Ok { version } => {
            let status = version.to_string();
            // 成功スタイルで出力
            let check_mark = if out.config().color { "\x1b[32m✓\x1b[0m" } else { "✓" };
            eprintln!("  {} {}: {}", check_mark, name, status);
            if verbose {
                if let Some(path) = &check.path {
                    eprintln!("    パス: {}", path);
                }
            }
        }
        CheckStatus::VersionMismatch { actual, required } => {
            eprintln!(
                "  {} {}: {} (必要: {} 以上)",
                if out.config().color { "\x1b[33m⚠\x1b[0m" } else { "⚠" },
                name,
                actual,
                required
            );
        }
        CheckStatus::NotFound => {
            let suffix = if check.requirement.required {
                ""
            } else {
                &format!(" ({}に必要)", check.requirement.description)
            };
            eprintln!(
                "  {} {}: not found{}",
                if check.requirement.required {
                    if out.config().color { "\x1b[31m✗\x1b[0m" } else { "✗" }
                } else if out.config().color {
                    "\x1b[90m-\x1b[0m"
                } else {
                    "-"
                },
                name,
                suffix
            );
        }
        CheckStatus::Error(msg) => {
            eprintln!(
                "  {} {}: エラー ({})",
                if out.config().color { "\x1b[31m✗\x1b[0m" } else { "✗" },
                name,
                msg
            );
        }
    }
}

// --- JSON 出力用構造体 ---

#[derive(Serialize)]
struct DoctorOutput {
    k1s0_version: String,
    checks: Vec<ToolCheckJson>,
    recommendations: Vec<RecommendationJson>,
    summary: SummaryJson,
}

#[derive(Serialize)]
struct ToolCheckJson {
    name: String,
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    required_version: Option<String>,
    required: bool,
    category: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<String>,
}

#[derive(Serialize)]
struct RecommendationJson {
    tool: String,
    action: String,
    message: String,
    url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    install_command: Option<String>,
    required: bool,
}

#[derive(Serialize)]
struct SummaryJson {
    required_ok: usize,
    required_failed: usize,
    optional_ok: usize,
    optional_missing: usize,
}

fn output_json(checks: &[ToolCheck], recommendations: &[Recommendation], summary: &CheckSummary) {
    let output = DoctorOutput {
        k1s0_version: crate::version().to_string(),
        checks: checks
            .iter()
            .map(|c| {
                let (status, version, required_version) = match &c.status {
                    CheckStatus::Ok { version } => ("ok".to_string(), Some(version.clone()), None),
                    CheckStatus::VersionMismatch { actual, required } => (
                        "version_mismatch".to_string(),
                        Some(actual.clone()),
                        Some(required.clone()),
                    ),
                    CheckStatus::NotFound => ("not_found".to_string(), None, None),
                    CheckStatus::Error(msg) => (format!("error: {}", msg), None, None),
                };
                ToolCheckJson {
                    name: c.requirement.name.to_string(),
                    status,
                    version,
                    required_version,
                    required: c.requirement.required,
                    category: c.requirement.category.name().to_string(),
                    path: c.path.clone(),
                }
            })
            .collect(),
        recommendations: recommendations
            .iter()
            .map(|r| RecommendationJson {
                tool: r.tool_name.to_string(),
                action: match r.action {
                    RecommendAction::Install => "install".to_string(),
                    RecommendAction::Upgrade => "upgrade".to_string(),
                },
                message: r.message.clone(),
                url: r.url.to_string(),
                install_command: r.install_command.map(|s| s.to_string()),
                required: r.required,
            })
            .collect(),
        summary: SummaryJson {
            required_ok: summary.required_ok,
            required_failed: summary.required_failed,
            optional_ok: summary.optional_ok,
            optional_missing: summary.optional_missing,
        },
    };

    if let Ok(json) = serde_json::to_string_pretty(&output) {
        println!("{}", json);
    }
}
