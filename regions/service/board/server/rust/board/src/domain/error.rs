// ボードドメインエラー型。
// Repository トレイトのインフラエラーをドメイン層に持ち込まないため、
// Infrastructure バリアントで anyhow::Error を包む。
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BoardError {
    #[error("WIP limit exceeded: column '{column_id}' has {current}/{limit} tasks")]
    WipLimitExceeded { column_id: String, current: i32, limit: i32 },
    #[error("version conflict: expected {expected}, got {actual}")]
    VersionConflict { expected: i32, actual: i32 },
    #[error("board column not found: {0}")]
    NotFound(String),
    #[error("validation failed: {0}")]
    ValidationFailed(String),
    /// インフラ層（DB・ネットワーク等）のエラーをドメイン型に包むバリアント
    #[error("infrastructure error: {0}")]
    Infrastructure(#[from] anyhow::Error),
}
