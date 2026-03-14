use thiserror::Error;

#[derive(Debug, Error)]
pub enum ComponentError {
    #[error("初期化エラー: {0}")]
    Init(String),
    #[error("設定エラー: {0}")]
    Config(String),
    #[error("ランタイムエラー: {0}")]
    Runtime(String),
    #[error("シャットダウンエラー: {0}")]
    Shutdown(String),
    #[error("コンポーネントが見つかりません: {0}")]
    NotFound(String),
}
