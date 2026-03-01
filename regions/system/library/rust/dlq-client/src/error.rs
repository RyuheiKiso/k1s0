use thiserror::Error;

/// DlqError は DLQ クライアントのエラー型。
#[derive(Debug, Error)]
pub enum DlqError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("API error (status {status}): {message}")]
    Api { status: u16, message: String },
    #[error("Deserialization error: {0}")]
    Deserialize(String),
}
