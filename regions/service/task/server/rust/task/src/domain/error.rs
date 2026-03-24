// タスクドメインエラー型。
// Repository トレイトのインフラエラーをドメイン層に持ち込まないため、
// Infrastructure バリアントで anyhow::Error を包む。
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TaskError {
    #[error("invalid status transition: from '{from}' to '{to}'")]
    InvalidStatusTransition { from: String, to: String },
    #[error("validation failed: {0}")]
    ValidationFailed(String),
    #[error("task not found: {0}")]
    NotFound(String),
    /// インフラ層（DB・ネットワーク等）のエラーをドメイン型に包むバリアント
    #[error("infrastructure error: {0}")]
    Infrastructure(#[from] anyhow::Error),
}
