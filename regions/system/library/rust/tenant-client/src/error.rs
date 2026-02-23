use thiserror::Error;

#[derive(Debug, Error)]
pub enum TenantError {
    #[error("テナントが見つかりません: {0}")]
    NotFound(String),
    #[error("テナントは停止中です: {0}")]
    Suspended(String),
    #[error("サーバーエラー: {0}")]
    ServerError(String),
    #[error("タイムアウト: {0}")]
    Timeout(String),
}
