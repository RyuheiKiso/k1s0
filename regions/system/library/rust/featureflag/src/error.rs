use thiserror::Error;

#[derive(Debug, Error)]
pub enum FeatureFlagError {
    #[error("フラグが見つかりません: {key}")]
    FlagNotFound { key: String },
    #[error("接続エラー: {0}")]
    ConnectionError(String),
    #[error("設定エラー: {0}")]
    ConfigError(String),
}
