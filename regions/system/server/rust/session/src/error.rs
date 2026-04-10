use thiserror::Error;

/// セッション操作に関するエラー型。
/// Expired: `セッションの有効期限切れ（get_session` usecase で返される）。
/// `TooManySessions`: `デバイス上限超過（create_session` usecase で将来返す予定）。
/// セッション操作に関するエラー型
#[derive(Debug, Error)]
pub enum SessionError {
    #[error("session not found: {0}")]
    NotFound(String),
    #[error("session expired: {0}")]
    Expired(String),
    #[error("session revoked: {0}")]
    Revoked(String),
    #[error("session already revoked: {0}")]
    AlreadyRevoked(String),
    #[error("invalid input: {0}")]
    InvalidInput(String),
    /// H-02 監査対応: 将来の同時セッション数制限機能（デバイス上限超過）実装に備えて保持する
    #[allow(dead_code)]
    #[error("too many sessions for user: {0}")]
    TooManySessions(String),
    #[error("internal error: {0}")]
    Internal(String),
}
