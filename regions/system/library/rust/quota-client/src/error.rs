use thiserror::Error;

#[derive(Debug, Error)]
pub enum QuotaClientError {
    #[error("接続エラー: {0}")]
    ConnectionError(String),
    #[error("クォータ超過: quota_id={quota_id}, remaining={remaining}")]
    QuotaExceeded { quota_id: String, remaining: u64 },
    #[error("クォータが見つかりません: {0}")]
    NotFound(String),
    #[error("無効なレスポンス: {0}")]
    InvalidResponse(String),
    #[error("内部エラー: {0}")]
    Internal(String),
}
