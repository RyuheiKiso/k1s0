use thiserror::Error;

/// レート制限クライアントのエラー型。
/// gRPC / HTTP 通信エラー、制限超過、キー不在、タイムアウトを表現する。
#[derive(Debug, Error)]
pub enum RateLimitError {
    /// `レート制限が超過した場合（retry_after_secs` 秒後に再試行可能）
    #[error("レート制限超過: {retry_after_secs}秒後に再試行してください")]
    LimitExceeded { retry_after_secs: u64 },
    /// 指定されたキーが見つからない場合
    #[error("キーが見つかりません: {key}")]
    KeyNotFound { key: String },
    /// サーバー側でエラーが発生した場合
    #[error("サーバーエラー: {0}")]
    ServerError(String),
    /// リクエストがタイムアウトした場合
    #[error("タイムアウト")]
    Timeout,
    /// サーバーへの接続に失敗した場合
    #[error("接続エラー: {0}")]
    ConnectionError(String),
}
