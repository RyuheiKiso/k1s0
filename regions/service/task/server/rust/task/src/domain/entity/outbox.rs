// アウトボックスエンティティ。Transactional Outbox パターン用。
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct OutboxEvent {
    pub id: Uuid,
    pub aggregate_id: Uuid,
    pub aggregate_type: String,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub published: bool,
    pub created_at: DateTime<Utc>,
}
