use thiserror::Error;

#[derive(Debug, Error)]
pub enum FileClientError {
    #[error("接続エラー: {0}")]
    ConnectionError(String),
    #[error("認証エラー: {0}")]
    Unauthorized(String),
    #[error("ファイルが見つかりません: {0}")]
    NotFound(String),
    #[error("クォータ超過: {0}")]
    QuotaExceeded(String),
    #[error("設定エラー: {0}")]
    InvalidConfig(String),
    #[error("内部エラー: {0}")]
    Internal(String),
}
