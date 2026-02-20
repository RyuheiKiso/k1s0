//! サービス間認証エラー定義。

/// ServiceAuthError はサービス間認証処理で発生するエラーを表す。
#[derive(thiserror::Error, Debug)]
pub enum ServiceAuthError {
    /// トークン取得に失敗した。
    #[error("トークン取得失敗: {0}")]
    TokenAcquisition(String),

    /// トークンの有効期限が切れた。
    #[error("トークンの有効期限が切れました")]
    TokenExpired,

    /// トークンが無効または検証に失敗した。
    #[error("無効なトークン: {0}")]
    InvalidToken(String),

    /// SPIFFE ID の検証に失敗した。
    #[error("SPIFFE ID 検証失敗: {0}")]
    SpiffeValidationFailed(String),

    /// OIDC ディスカバリーに失敗した。
    #[error("OIDC ディスカバリー失敗: {0}")]
    OidcDiscovery(String),

    /// HTTP リクエストに失敗した。
    #[error("HTTP リクエスト失敗: {0}")]
    Http(String),
}
