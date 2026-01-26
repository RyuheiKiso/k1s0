//! `k1s0 lint` コマンド
//!
//! 規約違反を検査する。
//!
//! # ルール ID
//!
//! - `K001`: manifest.json が存在しない
//! - `K002`: manifest.json の必須キーが不足
//! - `K003`: manifest.json の値が不正
//! - `K010`: 必須ディレクトリが存在しない
//! - `K011`: 必須ファイルが存在しない
//! - `K020`: 環境変数参照の禁止
//! - `K021`: config YAML への機密直書き禁止
//! - `K022`: Clean Architecture 依存方向違反

use std::path::PathBuf;

use clap::Args;
use k1s0_generator::lint::{LintConfig, LintResult, Linter, Severity};

use crate::error::{CliError, Result};
use crate::output::{output, LintOutput, LintViolation};

/// `k1s0 lint` の引数
#[derive(Args, Debug)]
pub struct LintArgs {
    /// 検査するディレクトリ（デフォルト: カレントディレクトリ）
    #[arg(default_value = ".")]
    pub path: String,

    /// 特定のルールのみ実行（カンマ区切り）
    #[arg(long)]
    pub rules: Option<String>,

    /// 特定のルールを除外（カンマ区切り）
    #[arg(long)]
    pub exclude_rules: Option<String>,

    /// 警告をエラーとして扱う
    #[arg(long)]
    pub strict: bool,

    /// 自動修正を試みる（未実装）
    #[arg(long)]
    pub fix: bool,

    /// JSON 形式で出力
    #[arg(long)]
    pub json: bool,

    /// 環境変数参照を許可するファイルパス（カンマ区切り、glob パターン対応）
    #[arg(long)]
    pub env_var_allowlist: Option<String>,
}

/// `k1s0 lint` を実行する
pub fn execute(args: LintArgs) -> Result<()> {
    let out = output();
    let path = PathBuf::from(&args.path);

    // パスの存在確認
    if !path.exists() {
        return Err(CliError::io("指定されたパスが存在しません")
            .with_target(&args.path)
            .with_hint("正しいパスを指定してください"));
    }

    // 設定の構築
    let config = LintConfig {
        rules: args.rules.map(|r| r.split(',').map(|s| s.trim().to_string()).collect()),
        exclude_rules: args
            .exclude_rules
            .map(|r| r.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default(),
        strict: args.strict,
        env_var_allowlist: args
            .env_var_allowlist
            .map(|r| r.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default(),
    };

    // lint 実行
    let linter = Linter::new(config);
    let result = linter.lint(&path);

    // JSON 出力
    if args.json {
        let output_json = to_lint_output(&result);
        out.print_json(&output_json);

        if result.is_success() {
            return Ok(());
        } else {
            return Err(CliError::validation("lint に失敗しました"));
        }
    }

    // 人間向け出力
    print_result(&result);

    if result.is_success() {
        Ok(())
    } else {
        Err(CliError::validation(format!(
            "lint に失敗しました（エラー: {}, 警告: {}）",
            result.error_count(),
            result.warning_count()
        )))
    }
}

/// 結果を出力する
fn print_result(result: &LintResult) {
    let out = output();

    out.header("k1s0 lint");
    out.newline();
    out.list_item("path", &result.path.display().to_string());
    out.newline();

    if result.violations.is_empty() {
        out.success("すべての検査に合格しました");
        return;
    }

    // 違反を出力
    out.header("違反:");
    out.newline();

    for v in &result.violations {
        let severity_prefix = match v.severity {
            Severity::Error => "error",
            Severity::Warning => "warn",
        };

        // [K001] error: message
        let msg = format!(
            "[{}] {}: {}",
            v.rule.as_str(),
            severity_prefix,
            v.message
        );

        match v.severity {
            Severity::Error => out.warning(&msg), // 赤色で表示
            Severity::Warning => out.info(&msg),  // 黄色で表示
        }

        // パス
        if let Some(path) = &v.path {
            out.hint(&format!("  対象: {}", path));
        }

        // ヒント
        if let Some(hint) = &v.hint {
            out.hint(&format!("  ヒント: {}", hint));
        }
    }

    out.newline();

    // サマリー
    let error_count = result.error_count();
    let warning_count = result.warning_count();

    let summary = format!(
        "エラー: {}, 警告: {}",
        error_count, warning_count
    );

    if error_count > 0 {
        out.warning(&format!("検査失敗 - {}", summary));
    } else {
        out.info(&format!("検査完了 - {}", summary));
    }
}

/// LintResult を LintOutput に変換する
fn to_lint_output(result: &LintResult) -> LintOutput {
    LintOutput {
        error: !result.is_success(),
        path: result.path.display().to_string(),
        violation_count: result.error_count(),
        warning_count: result.warning_count(),
        violations: result
            .violations
            .iter()
            .map(|v| LintViolation {
                rule: v.rule.as_str().to_string(),
                severity: v.severity.as_str().to_string(),
                message: v.message.clone(),
                path: v.path.clone(),
                line: v.line,
            })
            .collect(),
    }
}
