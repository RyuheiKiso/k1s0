// アクティビティドメインエラー型。
// Repository トレイトのインフラエラーをドメイン層に持ち込まないため、
// Infrastructure バリアントで anyhow::Error を包む。
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ActivityError {
    #[error("invalid status transition: from '{from}' to '{to}'")]
    InvalidStatusTransition { from: String, to: String },
    #[error("activity not found: {0}")]
    NotFound(String),
    #[error("validation failed: {0}")]
    ValidationFailed(String),
    #[error("idempotency key already used: {0}")]
    DuplicateIdempotencyKey(String),
    /// インフラ層（DB・ネットワーク等）のエラーをドメイン型に包むバリアント
    #[error("infrastructure error: {0}")]
    Infrastructure(#[from] anyhow::Error),
}
