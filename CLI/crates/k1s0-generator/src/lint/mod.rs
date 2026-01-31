//! lint ルールと検査
//!
//! k1s0 規約に対する検査を提供する。
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
//! - `K025`: 設定ファイル命名規約違反
//! - `K026`: Domain 層でのプロトコル依存検出
//! - `K030`: gRPC リトライ設定の検出（可視化）
//! - `K031`: gRPC リトライ設定に ADR 参照がない
//! - `K032`: gRPC リトライ設定が不完全
//! - `K040`: 層間依存の基本違反
//! - `K041`: domain が見つからない
//! - `K042`: domain バージョン制約不整合
//! - `K043`: 循環依存の検出
//! - `K044`: 非推奨 domain の使用
//! - `K045`: min_framework_version 違反
//! - `K046`: breaking_changes の影響
//! - `K047`: domain 層の version 未設定
//! - `K050`: SQL インジェクションリスク検出
//! - `K053`: ログへの機密情報出力検出
//! - `K028`: 未使用 domain 依存の検出
//! - `K029`: 本番コードでのパニック検出
//! - `K060`: Dockerfile ベースイメージ未固定
//!
//! # 機能
//!
//! - Watch モード: `--watch` フラグでファイル変更を監視し継続的に lint 実行
//! - 差分 lint: `--diff <base>` で変更ファイルのみを対象に lint 実行

mod config_naming;
mod dependency;
pub mod diff;
mod dockerfile_lint;
mod env_vars;
mod fixer;
mod layer_dependency;
mod linter;
mod panic_detection;
mod protocol_dependency;
mod required_files;
mod retry;
mod secret_config;
mod sensitive_logging;
mod sql_injection;
mod types;
mod unused_domain;
mod utils;
pub mod watch;

#[cfg(test)]
mod tests;

pub use dependency::DependencyRules;
pub use diff::{diff_from_head, diff_from_main, DiffError, DiffFilter, GitDiff};
pub use env_vars::{EnvVarPattern, EnvVarPatterns};
pub use fixer::Fixer;
pub use layer_dependency::LayerDependencyRules;
pub use linter::Linter;
pub use required_files::RequiredFiles;
pub use secret_config::SecretKeyPatterns;
pub use types::{FixResult, LintConfig, LintResult, RuleId, Severity, Violation};
pub use watch::{FileChangeEvent, FileChangeKind, LintWatcher, WatchConfig};

pub(crate) use utils::{contains_adr_reference, parse_yaml_line};
