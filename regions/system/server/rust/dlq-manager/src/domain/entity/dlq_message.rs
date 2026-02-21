use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

impl DlqStatus {
    pub fn from_str_value(s: &str) -> anyhow::Result<Self> {
        match s {
            "PENDING" => Ok(Self::Pending),
            "RETRYING" => Ok(Self::Retrying),
            "RESOLVED" => Ok(Self::Resolved),
            "DEAD" => Ok(Self::Dead),
            _ => anyhow::bail!("invalid DLQ status: {}", s),
        }
    }
}

/// DlqMessage は DLQ メッセージエンティティ。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DlqMessage {
    pub id: Uuid,
    pub original_topic: String,
    pub error_message: String,
    pub retry_count: i32,
    pub max_retries: i32,
    pub payload: serde_json::Value,
    pub status: DlqStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_retry_at: Option<DateTime<Utc>>,
}

impl DlqMessage {
    /// 新しい DlqMessage を作成する。
    pub fn new(
        original_topic: String,
        error_message: String,
        payload: serde_json::Value,
        max_retries: i32,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            original_topic,
            error_message,
            retry_count: 0,
            max_retries,
            payload,
            status: DlqStatus::Pending,
            created_at: now,
            updated_at: now,
            last_retry_at: None,
        }
    }

    /// リトライ中としてマークする。
    pub fn mark_retrying(&mut self) {
        self.status = DlqStatus::Retrying;
        self.retry_count += 1;
        self.last_retry_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// 解決済みとしてマークする。
    pub fn mark_resolved(&mut self) {
        self.status = DlqStatus::Resolved;
        self.updated_at = Utc::now();
    }

    /// デッド（処理不能）としてマークする。
    pub fn mark_dead(&mut self) {
        self.status = DlqStatus::Dead;
        self.updated_at = Utc::now();
    }

    /// リトライ可能かどうかを返す。
    pub fn is_retryable(&self) -> bool {
        matches!(self.status, DlqStatus::Pending | DlqStatus::Retrying)
            && self.retry_count < self.max_retries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_message() -> DlqMessage {
        DlqMessage::new(
            "orders.events.v1".to_string(),
            "processing failed".to_string(),
            serde_json::json!({"order_id": "123"}),
            3,
        )
    }

    #[test]
    fn test_new_message() {
        let msg = make_message();
        assert_eq!(msg.status, DlqStatus::Pending);
        assert_eq!(msg.retry_count, 0);
        assert_eq!(msg.max_retries, 3);
        assert_eq!(msg.original_topic, "orders.events.v1");
        assert!(msg.last_retry_at.is_none());
    }

    #[test]
    fn test_mark_retrying() {
        let mut msg = make_message();
        msg.mark_retrying();
        assert_eq!(msg.status, DlqStatus::Retrying);
        assert_eq!(msg.retry_count, 1);
        assert!(msg.last_retry_at.is_some());
    }

    #[test]
    fn test_mark_resolved() {
        let mut msg = make_message();
        msg.mark_resolved();
        assert_eq!(msg.status, DlqStatus::Resolved);
    }

    #[test]
    fn test_mark_dead() {
        let mut msg = make_message();
        msg.mark_dead();
        assert_eq!(msg.status, DlqStatus::Dead);
    }

    #[test]
    fn test_is_retryable_pending() {
        let msg = make_message();
        assert!(msg.is_retryable());
    }

    #[test]
    fn test_is_retryable_retrying() {
        let mut msg = make_message();
        msg.mark_retrying();
        assert!(msg.is_retryable());
    }

    #[test]
    fn test_is_retryable_resolved() {
        let mut msg = make_message();
        msg.mark_resolved();
        assert!(!msg.is_retryable());
    }

    #[test]
    fn test_is_retryable_dead() {
        let mut msg = make_message();
        msg.mark_dead();
        assert!(!msg.is_retryable());
    }

    #[test]
    fn test_is_retryable_max_retries_exceeded() {
        let mut msg = make_message();
        msg.mark_retrying();
        msg.mark_retrying();
        msg.mark_retrying();
        // retry_count == 3, max_retries == 3
        assert!(!msg.is_retryable());
    }

    #[test]
    fn test_status_display() {
        assert_eq!(DlqStatus::Pending.to_string(), "PENDING");
        assert_eq!(DlqStatus::Retrying.to_string(), "RETRYING");
        assert_eq!(DlqStatus::Resolved.to_string(), "RESOLVED");
        assert_eq!(DlqStatus::Dead.to_string(), "DEAD");
    }

    #[test]
    fn test_status_from_str() {
        assert_eq!(DlqStatus::from_str_value("PENDING").unwrap(), DlqStatus::Pending);
        assert_eq!(DlqStatus::from_str_value("RETRYING").unwrap(), DlqStatus::Retrying);
        assert_eq!(DlqStatus::from_str_value("RESOLVED").unwrap(), DlqStatus::Resolved);
        assert_eq!(DlqStatus::from_str_value("DEAD").unwrap(), DlqStatus::Dead);
        assert!(DlqStatus::from_str_value("INVALID").is_err());
    }

    #[test]
    fn test_mark_retrying_increments_count() {
        let mut msg = make_message();
        msg.mark_retrying();
        assert_eq!(msg.retry_count, 1);
        msg.mark_retrying();
        assert_eq!(msg.retry_count, 2);
        msg.mark_retrying();
        assert_eq!(msg.retry_count, 3);
    }

    #[test]
    fn test_new_message_has_uuid() {
        let msg1 = make_message();
        let msg2 = make_message();
        assert_ne!(msg1.id, msg2.id);
    }
}
