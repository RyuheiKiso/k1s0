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
//! - `K030`: gRPC リトライ設定の検出（可視化）
//! - `K031`: gRPC リトライ設定に ADR 参照がない
//! - `K032`: gRPC リトライ設定が不完全

mod dependency;
mod env_vars;
mod fixer;
mod linter;
mod required_files;
mod retry;
mod secret_config;
mod types;
mod utils;

#[cfg(test)]
mod tests;

pub use dependency::DependencyRules;
pub use env_vars::{EnvVarPattern, EnvVarPatterns};
pub use fixer::Fixer;
pub use linter::Linter;
pub use required_files::RequiredFiles;
pub use secret_config::SecretKeyPatterns;
pub use types::{FixResult, LintConfig, LintResult, RuleId, Severity, Violation};

pub(crate) use utils::{contains_adr_reference, parse_yaml_line};
