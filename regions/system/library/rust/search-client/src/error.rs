use thiserror::Error;

#[derive(Debug, Error)]
pub enum SearchError {
    #[error("インデックスが見つかりません: {0}")]
    IndexNotFound(String),
    #[error("無効なクエリ: {0}")]
    InvalidQuery(String),
    #[error("サーバーエラー: {0}")]
    ServerError(String),
    #[error("タイムアウト")]
    Timeout,
}
