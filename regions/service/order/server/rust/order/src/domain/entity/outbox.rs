use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Outbox イベントエンティティ。
///
/// DB トランザクション内でビジネス操作と同時に書き込み、
/// 後続の OutboxPoller が Kafka へ確実にパブリッシュする。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboxEvent {
    pub id: Uuid,
    pub aggregate_type: String,
    pub aggregate_id: String,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub published_at: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_outbox_event_creation() {
        let event = OutboxEvent {
            id: Uuid::new_v4(),
            aggregate_type: "order".to_string(),
            aggregate_id: Uuid::new_v4().to_string(),
            event_type: "order.created".to_string(),
            payload: serde_json::json!({"order_id": "test"}),
            created_at: Utc::now(),
            published_at: None,
        };
        assert!(event.published_at.is_none());
        assert_eq!(event.aggregate_type, "order");
    }

    #[test]
    fn test_outbox_event_serialization_roundtrip() {
        let event = OutboxEvent {
            id: Uuid::new_v4(),
            aggregate_type: "order".to_string(),
            aggregate_id: "ORD-001".to_string(),
            event_type: "order.created".to_string(),
            payload: serde_json::json!({"key": "value"}),
            created_at: Utc::now(),
            published_at: None,
        };
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: OutboxEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.aggregate_id, "ORD-001");
        assert_eq!(deserialized.event_type, "order.created");
    }
}
