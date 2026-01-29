//! コンソール出力ユーティリティ
//!
//! 人間向けの最小フォーマットと、将来の `--json` 拡張に対応した出力機能を提供する。

use console::{style, Style, Term};
use indicatif::{ProgressBar, ProgressStyle};
use serde::Serialize;

use crate::error::CliError;

/// 出力モード
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputMode {
    /// 人間向けのフォーマット（デフォルト）
    #[default]
    Human,
    /// JSON フォーマット
    Json,
    /// 出力なし（テスト用）
    Quiet,
}

/// 出力設定
#[derive(Debug, Clone)]
pub struct OutputConfig {
    /// 出力モード
    pub mode: OutputMode,
    /// カラー出力を有効にするか
    pub color: bool,
    /// 詳細出力を有効にするか
    pub verbose: bool,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            mode: OutputMode::Human,
            color: true,
            verbose: false,
        }
    }
}

/// コンソール出力を担当する構造体
pub struct Output {
    config: OutputConfig,
    #[allow(dead_code)]
    term: Term,
}

impl Output {
    /// 新しい Output を作成
    pub fn new(config: OutputConfig) -> Self {
        Self {
            config,
            term: Term::stderr(),
        }
    }

    /// デフォルト設定で作成
    pub fn default_output() -> Self {
        Self::new(OutputConfig::default())
    }

    /// 設定を取得
    pub fn config(&self) -> &OutputConfig {
        &self.config
    }

    /// JSON モードかどうかを返す
    pub fn is_json_mode(&self) -> bool {
        self.config.mode == OutputMode::Json
    }

    /// カラー出力が有効かどうか
    fn use_color(&self) -> bool {
        self.config.color && self.config.mode == OutputMode::Human
    }

    // --- スタイル ---

    fn style_success(&self) -> Style {
        if self.use_color() {
            Style::new().green().bold()
        } else {
            Style::new()
        }
    }

    fn style_error(&self) -> Style {
        if self.use_color() {
            Style::new().red().bold()
        } else {
            Style::new()
        }
    }

    fn style_warning(&self) -> Style {
        if self.use_color() {
            Style::new().yellow().bold()
        } else {
            Style::new()
        }
    }

    fn style_info(&self) -> Style {
        if self.use_color() {
            Style::new().cyan()
        } else {
            Style::new()
        }
    }

    fn style_hint(&self) -> Style {
        if self.use_color() {
            Style::new().dim()
        } else {
            Style::new()
        }
    }

    fn style_path(&self) -> Style {
        if self.use_color() {
            Style::new().underlined()
        } else {
            Style::new()
        }
    }

    // --- 出力メソッド ---

    /// 成功メッセージを出力
    pub fn success(&self, message: &str) {
        if self.config.mode == OutputMode::Quiet {
            return;
        }

        let prefix = self.style_success().apply_to("✓");
        eprintln!("{} {}", prefix, message);
    }

    /// エラーメッセージを出力
    pub fn error(&self, error: &CliError) {
        if self.config.mode == OutputMode::Json {
            self.print_json(&ErrorOutput::from(error));
            return;
        }

        if self.config.mode == OutputMode::Quiet {
            return;
        }

        // エラー種別とメッセージ
        let prefix = self.style_error().apply_to("✗");
        let kind_label = self.style_error().apply_to(error.kind.label());
        eprintln!("{} {}: {}", prefix, kind_label, error.message);

        // 対象
        if let Some(target) = &error.target {
            let path_styled = self.style_path().apply_to(target);
            eprintln!("  対象: {}", path_styled);
        }

        // 次のアクション
        if let Some(hint) = &error.hint {
            let hint_styled = self.style_hint().apply_to(hint);
            eprintln!("  ヒント: {}", hint_styled);
        }

        // リカバリコマンド
        if !error.recovery_commands.is_empty() {
            eprintln!();
            let recovery_label = self.style_info().apply_to("試してみてください:");
            eprintln!("  {}", recovery_label);
            for rc in &error.recovery_commands {
                let cmd_styled = if self.use_color() {
                    style(&rc.command).green().bold().to_string()
                } else {
                    rc.command.clone()
                };
                eprintln!("    $ {}  # {}", cmd_styled, rc.description);
            }
        }

        // 詳細（verbose モード）
        if self.config.verbose {
            if let Some(source) = &error.source {
                eprintln!("  詳細: {}", source);
            }
        }
    }

    /// 警告メッセージを出力
    pub fn warning(&self, message: &str) {
        if self.config.mode == OutputMode::Quiet {
            return;
        }

        let prefix = self.style_warning().apply_to("⚠");
        eprintln!("{} {}", prefix, message);
    }

    /// 情報メッセージを出力
    pub fn info(&self, message: &str) {
        if self.config.mode == OutputMode::Quiet {
            return;
        }

        let prefix = self.style_info().apply_to("ℹ");
        eprintln!("{} {}", prefix, message);
    }

    /// ヒントを出力
    pub fn hint(&self, message: &str) {
        if self.config.mode == OutputMode::Quiet {
            return;
        }

        let hint_styled = self.style_hint().apply_to(message);
        eprintln!("  {}", hint_styled);
    }

    /// 詳細メッセージを出力（verbose モードのみ）
    pub fn verbose(&self, message: &str) {
        if !self.config.verbose || self.config.mode == OutputMode::Quiet {
            return;
        }

        let prefix = self.style_hint().apply_to("▸");
        eprintln!("{} {}", prefix, message);
    }

    /// 空行を出力
    pub fn newline(&self) {
        if self.config.mode == OutputMode::Quiet {
            return;
        }
        eprintln!();
    }

    /// ヘッダーを出力
    pub fn header(&self, title: &str) {
        if self.config.mode == OutputMode::Quiet {
            return;
        }

        if self.use_color() {
            eprintln!("{}", style(title).bold());
        } else {
            eprintln!("{}", title);
        }
    }

    /// リスト項目を出力
    pub fn list_item(&self, label: &str, value: &str) {
        if self.config.mode == OutputMode::Quiet {
            return;
        }

        let label_styled = if self.use_color() {
            style(label).dim().to_string()
        } else {
            label.to_string()
        };
        eprintln!("  {}: {}", label_styled, value);
    }

    /// ファイル追加を出力
    pub fn file_added(&self, path: &str) {
        if self.config.mode == OutputMode::Quiet {
            return;
        }

        let prefix = self.style_success().apply_to("+");
        let path_styled = self.style_path().apply_to(path);
        eprintln!("  {} {}", prefix, path_styled);
    }

    /// ファイル変更を出力
    pub fn file_modified(&self, path: &str) {
        if self.config.mode == OutputMode::Quiet {
            return;
        }

        let prefix = self.style_warning().apply_to("~");
        let path_styled = self.style_path().apply_to(path);
        eprintln!("  {} {}", prefix, path_styled);
    }

    /// ファイル削除を出力
    pub fn file_removed(&self, path: &str) {
        if self.config.mode == OutputMode::Quiet {
            return;
        }

        let prefix = self.style_error().apply_to("-");
        let path_styled = self.style_path().apply_to(path);
        eprintln!("  {} {}", prefix, path_styled);
    }

    /// ファイル衝突を出力
    pub fn file_conflict(&self, path: &str) {
        if self.config.mode == OutputMode::Quiet {
            return;
        }

        let prefix = self.style_error().apply_to("!");
        let path_styled = self.style_path().apply_to(path);
        eprintln!("  {} {} (衝突)", prefix, path_styled);
    }

    /// JSON を出力
    pub fn print_json<T: Serialize>(&self, value: &T) {
        if let Ok(json) = serde_json::to_string_pretty(value) {
            println!("{}", json);
        }
    }

    /// プログレスバーを作成
    pub fn progress_bar(&self, len: u64, message: &str) -> ProgressBar {
        if self.config.mode != OutputMode::Human {
            return ProgressBar::hidden();
        }

        let pb = ProgressBar::new(len);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("=>-"),
        );
        pb.set_message(message.to_string());
        pb
    }

    /// プレビューヘッダーを出力
    pub fn preview_header(&self, title: &str) {
        if self.config.mode == OutputMode::Quiet {
            return;
        }

        let prefix = self.style_info().apply_to("プレビュー:");
        eprintln!("{} {}", prefix, title);
    }

    /// プレビューサマリーを出力
    pub fn preview_summary(&self, file_count: usize, dir_count: usize) {
        if self.config.mode == OutputMode::Quiet {
            return;
        }

        eprintln!(
            "  {} ファイル, {} ディレクトリが生成されます",
            file_count, dir_count
        );
    }

    /// 続行確認を表示（inquire を使用）
    ///
    /// true を返した場合は続行、false はキャンセル
    pub fn confirm_proceed(&self, message: &str) -> bool {
        if self.config.mode != OutputMode::Human {
            return true;
        }

        inquire::Confirm::new(message)
            .with_default(true)
            .prompt()
            .unwrap_or_default()
    }

    /// スピナーを作成
    pub fn spinner(&self, message: &str) -> ProgressBar {
        if self.config.mode != OutputMode::Human {
            return ProgressBar::hidden();
        }

        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap(),
        );
        pb.set_message(message.to_string());
        pb.enable_steady_tick(std::time::Duration::from_millis(100));
        pb
    }
}

// --- JSON 出力用の構造体 ---

/// エラー出力（JSON 用）
#[derive(Debug, Serialize)]
pub struct ErrorOutput {
    /// エラーかどうか
    pub error: bool,
    /// エラーコード
    pub code: i32,
    /// エラー種別
    pub kind: String,
    /// メッセージ
    pub message: String,
    /// 対象
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    /// ヒント
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
    /// リカバリコマンド
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub recovery_commands: Vec<crate::error::RecoveryCommand>,
}

impl From<&CliError> for ErrorOutput {
    fn from(e: &CliError) -> Self {
        Self {
            error: true,
            code: e.exit_code().as_i32(),
            kind: format!("{:?}", e.kind),
            message: e.message.clone(),
            target: e.target.clone(),
            hint: e.hint.clone(),
            recovery_commands: e.recovery_commands.clone(),
        }
    }
}

/// 成功出力（JSON 用）
#[derive(Debug, Serialize)]
pub struct SuccessOutput<T: Serialize> {
    /// エラーかどうか
    pub error: bool,
    /// データ
    pub data: T,
}

impl<T: Serialize> SuccessOutput<T> {
    /// 新しい成功出力を作成
    pub fn new(data: T) -> Self {
        Self { error: false, data }
    }
}

/// lint 結果出力（JSON 用）
#[derive(Debug, Serialize)]
pub struct LintOutput {
    /// エラーかどうか
    pub error: bool,
    /// 検査したパス
    pub path: String,
    /// 違反の数
    pub violation_count: usize,
    /// 警告の数
    pub warning_count: usize,
    /// 違反リスト
    pub violations: Vec<LintViolation>,
}

/// lint 違反（JSON 用）
#[derive(Debug, Serialize)]
pub struct LintViolation {
    /// ルール ID
    pub rule: String,
    /// 重要度（error / warning）
    pub severity: String,
    /// メッセージ
    pub message: String,
    /// 対象パス
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// 行番号
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
}

// --- グローバル出力インスタンス ---

use std::sync::OnceLock;

static OUTPUT: OnceLock<Output> = OnceLock::new();

/// グローバル出力インスタンスを初期化
pub fn init_output(config: OutputConfig) {
    let _ = OUTPUT.set(Output::new(config));
}

/// グローバル出力インスタンスを取得
pub fn output() -> &'static Output {
    OUTPUT.get_or_init(Output::default_output)
}
