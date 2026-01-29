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
//! - `K030`: gRPC リトライ設定の検出（可視化）
//! - `K031`: gRPC リトライ設定に ADR 参照がない
//! - `K032`: gRPC リトライ設定が不完全
//!
//! # 追加機能
//!
//! - `--watch`: ファイル変更を監視して継続的に lint 実行
//! - `--diff <base>`: Git 差分で変更されたファイルのみを対象に lint 実行

use std::path::PathBuf;
#[cfg(feature = "watch")]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(feature = "watch")]
use std::sync::Arc;

use clap::Args;
use k1s0_generator::lint::{DiffFilter, Fixer, LintConfig, LintResult, Linter, Severity};
#[cfg(feature = "watch")]
use k1s0_generator::lint::{LintWatcher, WatchConfig};

use crate::error::{CliError, Result};
use crate::output::{output, LintOutput, LintViolation};
use crate::settings::Settings;

/// `k1s0 lint` の引数
#[derive(Args, Debug)]
#[command(after_long_help = r#"例:
  k1s0 lint
  k1s0 lint --rules K020,K021,K022 --strict
  k1s0 lint --fix --diff main

K010/K011 違反が検出された場合は 'k1s0 lint --fix' で自動修正できます。
"#)]
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

    /// 自動修正を試みる（K001, K002, K010, K011 に対応）
    #[arg(long)]
    pub fix: bool,

    /// JSON 形式で出力
    #[arg(long)]
    pub json: bool,

    /// 環境変数参照を許可するファイルパス（カンマ区切り、glob パターン対応）
    #[arg(long)]
    pub env_var_allowlist: Option<String>,

    /// ファイル変更を監視して継続的に lint 実行（Ctrl+C で終了）
    #[arg(long)]
    pub watch: bool,

    /// Git 差分で変更されたファイルのみを対象に lint 実行
    /// 例: --diff HEAD, --diff main, --diff origin/main
    #[arg(long)]
    pub diff: Option<String>,

    /// デバウンス間隔（ミリ秒、--watch と併用）
    #[arg(long, default_value = "500")]
    pub debounce_ms: u64,

    /// 設定ファイルを使用しない
    #[arg(long)]
    pub no_config: bool,
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

    // 設定ファイルを読み込み（--no-config でない場合）
    let settings = if args.no_config {
        Settings::default()
    } else {
        Settings::load(Some(&path)).unwrap_or_default()
    };

    // 設定の構築（CLI 引数 > 設定ファイル）
    let config = LintConfig {
        rules: args
            .rules
            .as_ref()
            .map(|r| r.split(',').map(|s| s.trim().to_string()).collect())
            .or(settings.lint.rules),
        exclude_rules: args
            .exclude_rules
            .as_ref()
            .map(|r| r.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_else(|| settings.lint.exclude_rules.clone()),
        strict: args.strict || settings.lint.strict,
        env_var_allowlist: args
            .env_var_allowlist
            .as_ref()
            .map(|r| r.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_else(|| settings.lint.env_var_allowlist.clone()),
        fix: args.fix || settings.lint.fix,
    };

    // デバウンス間隔（CLI 引数 > 設定ファイル）
    let debounce_ms = if args.debounce_ms != 500 {
        args.debounce_ms
    } else {
        settings.lint.watch_debounce_ms
    };

    // Watch モード
    if args.watch {
        return execute_watch(&path, config, debounce_ms, args.json);
    }

    // lint 実行
    let spinner = out.spinner("lint 実行中...");
    let linter = Linter::new(config.clone());
    let mut result = linter.lint(&path);
    spinner.finish_and_clear();

    // 差分フィルタ
    if let Some(base) = &args.diff {
        let filter = DiffFilter::new(&path);

        match filter.get_diff(base) {
            Ok(diff) => {
                out.info(&format!(
                    "差分フィルタ: {} との比較（変更ファイル: {} 件）",
                    base,
                    diff.changed_files.len() + diff.added_files.len()
                ));
                out.newline();
                result = filter.filter_violations(&result, &diff);
            }
            Err(e) => {
                out.warning(&format!("差分取得に失敗: {}", e));
                out.hint("Git リポジトリで正しいベースブランチ/コミットを指定してください");
                out.newline();
            }
        }
    }

    // --fix が指定されている場合、自動修正を試みる
    if args.fix && !result.violations.is_empty() {
        let fixer = Fixer::new(&path);
        let mut fixed_count = 0;
        let mut fix_results = Vec::new();

        for violation in &result.violations {
            if Fixer::is_fixable(violation.rule) {
                if let Some(fix_result) = fixer.fix(violation) {
                    if fix_result.success {
                        fixed_count += 1;
                    }
                    fix_results.push(fix_result);
                }
            }
        }

        // 修正結果を出力
        if !fix_results.is_empty() {
            out.newline();
            out.header("自動修正:");
            out.newline();

            for fix_result in &fix_results {
                if fix_result.success {
                    out.success(&format!("  ✓ {}", fix_result.description));
                } else {
                    out.warning(&format!(
                        "  ✗ {} - {}",
                        fix_result.description,
                        fix_result.error.as_deref().unwrap_or("不明なエラー")
                    ));
                }
            }

            out.newline();
            out.info(&format!("{} 件の修正を適用しました", fixed_count));
        }

        // 修正後に再度 lint を実行
        if fixed_count > 0 {
            out.newline();
            out.info("修正後の再検査を実行中...");
            out.newline();

            let result = linter.lint(&path);

            if args.json {
                let output_json = to_lint_output(&result);
                out.print_json(&output_json);
            } else {
                print_result(&result);
            }

            if result.is_success() {
                return Ok(());
            } else {
                return Err(CliError::validation(format!(
                    "lint に失敗しました（エラー: {}, 警告: {}）",
                    result.error_count(),
                    result.warning_count()
                )));
            }
        }
    }

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
        let mut err = CliError::validation(format!(
            "lint に失敗しました（エラー: {}, 警告: {}）",
            result.error_count(),
            result.warning_count()
        ));

        // K010/K011 が含まれている場合は --fix を提案
        let has_fixable = result.violations.iter().any(|v| {
            let rule_str = v.rule.as_str();
            rule_str == "K010" || rule_str == "K011"
        });
        if has_fixable && !args.fix {
            err = err.with_recovery("k1s0 lint --fix", "自動修正可能な違反を修正");
        }

        Err(err)
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

/// Watch モードで lint を実行
#[cfg(feature = "watch")]
fn execute_watch(
    path: &PathBuf,
    config: LintConfig,
    debounce_ms: u64,
    json: bool,
) -> Result<()> {
    let out = output();

    out.header("k1s0 lint (watch mode)");
    out.newline();
    out.info(&format!("監視中: {}", path.display()));
    out.hint("Ctrl+C で終了");
    out.newline();

    // Ctrl+C ハンドラ
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    if ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .is_err()
    {
        out.warning("Ctrl+C ハンドラの設定に失敗しました");
    }

    // Watch 設定
    let watch_config = WatchConfig {
        debounce_ms,
        ..WatchConfig::default()
    };

    let watcher = LintWatcher::new(path, config).with_watch_config(watch_config);

    let running_ref = running.clone();

    watcher
        .watch(move |result, event| {
            // Ctrl+C が押されたら終了
            if !running_ref.load(Ordering::SeqCst) {
                return false;
            }

            // 変更イベントの表示
            if let Some(ev) = event {
                let out = output();
                out.newline();
                out.info(&format!(
                    "ファイル変更検出: {} 件",
                    ev.paths.len()
                ));
                for p in ev.paths.iter().take(5) {
                    out.hint(&format!("  - {}", p.display()));
                }
                if ev.paths.len() > 5 {
                    out.hint(&format!("  ... 他 {} 件", ev.paths.len() - 5));
                }
                out.newline();
            }

            // 結果の表示
            if json {
                let output_json = to_lint_output(&result);
                let out = output();
                out.print_json(&output_json);
            } else {
                print_result(&result);
            }

            true // 継続
        })
        .map_err(|e| CliError::internal(format!("Watch エラー: {}", e)))?;

    out.newline();
    out.info("監視を終了しました");

    Ok(())
}

/// Watch モードで lint を実行（watch feature が無効な場合）
#[cfg(not(feature = "watch"))]
fn execute_watch(
    _path: &PathBuf,
    _config: LintConfig,
    _debounce_ms: u64,
    _json: bool,
) -> Result<()> {
    Err(CliError::internal(
        "Watch モードは利用できません。`--features watch` でビルドしてください",
    ))
}
