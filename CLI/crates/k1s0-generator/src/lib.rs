//! k1s0-generator
//!
//! テンプレート展開・差分適用ライブラリ。
//!
//! # モジュール
//!
//! - `manifest`: manifest.json の読み書き
//! - `template`: テンプレートのレンダリング
//! - `fingerprint`: テンプレートの fingerprint 算出
//! - `diff`: 差分計算・表示
//! - `fs`: ファイル操作ユーティリティ
//! - `walker`: ディレクトリ走査ユーティリティ

pub mod diff;
pub mod fingerprint;
pub mod fs;
pub mod lint;
pub mod manifest;
pub mod template;
pub mod upgrade;
pub mod walker;

// Tera の Context を再エクスポート
pub use tera::Context;

/// k1s0-generator のエラー型
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// IO エラー
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON パースエラー
    #[error("JSON parse error: {0}")]
    JsonParse(#[from] serde_json::Error),

    /// YAML パースエラー
    #[error("YAML parse error: {0}")]
    YamlParse(#[from] serde_yaml::Error),

    /// テンプレートエラー
    #[error("Template error: {0}")]
    Template(#[from] tera::Error),

    /// manifest が見つからない
    #[error("Manifest not found: {0}")]
    ManifestNotFound(String),

    /// manifest のバリデーションエラー
    #[error("Manifest validation error: {0}")]
    ManifestValidation(String),

    /// テンプレートが見つからない
    #[error("Template not found: {0}")]
    TemplateNotFound(String),

    /// ファイルの衝突
    #[error("File conflict: {0}")]
    FileConflict(String),

    /// その他のエラー
    #[error("{0}")]
    Other(String),
}

/// Result 型エイリアス
pub type Result<T> = std::result::Result<T, Error>;
