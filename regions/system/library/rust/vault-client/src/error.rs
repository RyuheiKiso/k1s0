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
    /// Vault サーバーへの接続が確立できない場合のエラー。
    /// `vault_required=false` の場合にフォールバック判定で使用される。
    /// ネットワーク障害、DNS 解決失敗、サーバー未起動などが原因となる。
    #[error("Vault 接続不可: {0}")]
    ConnectionUnavailable(String),
}
