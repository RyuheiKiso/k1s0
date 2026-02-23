use thiserror::Error;

#[derive(Debug, Error)]
pub enum VaultError {
    #[error("シークレットが見つかりません: {0}")]
    NotFound(String),
    #[error("権限が拒否されました: {0}")]
    PermissionDenied(String),
    #[error("サーバーエラー: {0}")]
    ServerError(String),
    #[error("タイムアウト")]
    Timeout,
    #[error("リースが期限切れです: {0}")]
    LeaseExpired(String),
}
