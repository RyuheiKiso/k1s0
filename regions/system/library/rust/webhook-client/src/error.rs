use thiserror::Error;

#[derive(Debug, Error)]
pub enum WebhookError {
    #[error("リクエスト送信エラー: {0}")]
    RequestFailed(String),
    #[error("シリアライズエラー: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("署名生成エラー: {0}")]
    SignatureError(String),
    #[error("内部エラー: {0}")]
    Internal(String),
    #[error("リトライ上限到達: {attempts}回試行, 最終ステータス={last_status_code}")]
    MaxRetriesExceeded {
        attempts: u32,
        last_status_code: u16,
    },
}
