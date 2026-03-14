use thiserror::Error;

#[derive(Debug, Error)]
pub enum BindingError {
    #[error("バインディング呼び出しエラー: {0}")]
    Invoke(String),
    #[error("読み取りエラー: {0}")]
    Read(String),
    #[error("サポートされていない操作: {0}")]
    UnsupportedOperation(String),
    #[error("接続エラー: {0}")]
    Connection(String),
    #[error("コンポーネントエラー: {0}")]
    Component(#[from] k1s0_bb_core::ComponentError),
}
