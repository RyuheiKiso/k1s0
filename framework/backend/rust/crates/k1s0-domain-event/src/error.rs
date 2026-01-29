use thiserror::Error;

/// イベント発行時のエラー。
#[derive(Debug, Error)]
pub enum PublishError {
    /// シリアライズに失敗した。
    #[error("failed to serialize event: {0}")]
    Serialization(#[from] serde_json::Error),

    /// バスが閉じている等、送信に失敗した。
    #[error("failed to send event: {0}")]
    Send(String),

    /// その他の内部エラー。
    #[error("publish error: {0}")]
    Internal(String),
}

/// イベント購読時のエラー。
#[derive(Debug, Error)]
pub enum SubscribeError {
    /// 指定されたイベント型が不正。
    #[error("invalid event type: {0}")]
    InvalidEventType(String),

    /// その他の内部エラー。
    #[error("subscribe error: {0}")]
    Internal(String),
}

/// イベントハンドラ内のエラー。
#[derive(Debug, Error)]
pub enum HandlerError {
    /// ハンドラのビジネスロジックエラー。
    #[error("handler failed: {0}")]
    Failed(String),

    /// リトライ可能な一時エラー。
    #[error("transient handler error: {0}")]
    Transient(String),
}

/// Outbox 操作時のエラー。
#[derive(Debug, Error)]
pub enum OutboxError {
    /// データベース操作に失敗した。
    #[error("outbox database error: {0}")]
    Database(String),

    /// シリアライズ/デシリアライズに失敗した。
    #[error("outbox serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// その他の内部エラー。
    #[error("outbox error: {0}")]
    Internal(String),
}
