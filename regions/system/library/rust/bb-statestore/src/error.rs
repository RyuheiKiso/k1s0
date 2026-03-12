use thiserror::Error;

#[derive(Debug, Error)]
pub enum StateStoreError {
    #[error("キーが見つかりません: {0}")]
    NotFound(String),
    #[error("ETag が一致しません: expected={expected}, actual={actual}")]
    ETagMismatch { expected: String, actual: String },
    #[error("接続エラー: {0}")]
    Connection(String),
    #[error("シリアライズエラー: {0}")]
    Serialization(String),
    #[error("コンポーネントエラー: {0}")]
    Component(#[from] k1s0_building_blocks::ComponentError),
}
