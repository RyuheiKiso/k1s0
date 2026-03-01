use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// DDD ドメインイベントトレイト。
/// すべてのドメインイベントはこのトレイトを実装する。
pub trait DomainEvent: Send + Sync + 'static {
    /// イベントの種類を返す。
    fn event_type(&self) -> &str;
    /// 集約IDを返す。
    fn aggregate_id(&self) -> &str;
    /// イベント発生日時を返す。
    fn occurred_at(&self) -> DateTime<Utc>;
}

/// 基本的なイベント構造体（後方互換性のため維持）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: Uuid,
    pub event_type: String,
    pub aggregate_id: String,
    pub payload: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

impl Event {
    pub fn new(event_type: String, payload: serde_json::Value) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_type,
            aggregate_id: String::new(),
            payload,
            timestamp: Utc::now(),
        }
    }

    /// 集約IDを指定してイベントを生成する。
    pub fn with_aggregate_id(
        event_type: String,
        aggregate_id: String,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_type,
            aggregate_id,
            payload,
            timestamp: Utc::now(),
        }
    }
}

impl DomainEvent for Event {
    fn event_type(&self) -> &str {
        &self.event_type
    }

    fn aggregate_id(&self) -> &str {
        &self.aggregate_id
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.timestamp
    }
}
