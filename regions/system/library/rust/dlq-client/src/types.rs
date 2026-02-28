use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// DlqStatus は DLQ メッセージのステータス。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DlqStatus {
    Pending,
    Retrying,
    Resolved,
    Dead,
}

impl std::fmt::Display for DlqStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "PENDING"),
            Self::Retrying => write!(f, "RETRYING"),
            Self::Resolved => write!(f, "RESOLVED"),
            Self::Dead => write!(f, "DEAD"),
        }
    }
}

/// DlqMessage は DLQ メッセージ。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DlqMessage {
    pub id: String,
    pub original_topic: String,
    pub error_message: String,
    pub retry_count: u32,
    pub max_retries: u32,
    pub payload: serde_json::Value,
    pub status: DlqStatus,
    pub created_at: DateTime<Utc>,
    pub last_retry_at: Option<DateTime<Utc>>,
}

/// ListDlqMessagesResponse は DLQ メッセージ一覧取得レスポンス。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListDlqMessagesResponse {
    pub messages: Vec<DlqMessage>,
    pub total: u32,
    pub page: u32,
}

/// RetryDlqMessageResponse は DLQ メッセージ再処理レスポンス。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryDlqMessageResponse {
    pub message_id: String,
    pub status: DlqStatus,
}
