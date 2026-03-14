use thiserror::Error;

#[derive(Debug, Error)]
pub enum SecretStoreError {
    #[error("シークレットが見つかりません: {0}")]
    NotFound(String),
    #[error("アクセス拒否: {0}")]
    PermissionDenied(String),
    #[error("接続エラー: {0}")]
    Connection(String),
    #[error("認証エラー: {0}")]
    Authentication(String),
    #[error("コンポーネントエラー: {0}")]
    Component(#[from] k1s0_bb_core::ComponentError),
}
