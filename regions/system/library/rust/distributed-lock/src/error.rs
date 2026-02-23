use thiserror::Error;

#[derive(Debug, Error)]
pub enum LockError {
    #[error("既にロック済みです: {0}")]
    AlreadyLocked(String),
    #[error("ロックが見つかりません: {0}")]
    LockNotFound(String),
    #[error("トークンが一致しません")]
    TokenMismatch,
    #[error("内部エラー: {0}")]
    Internal(String),
}
