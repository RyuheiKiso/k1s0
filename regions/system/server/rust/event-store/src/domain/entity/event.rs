use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    pub actor_id: Option<String>,
    pub correlation_id: Option<String>,
    pub causation_id: Option<String>,
}

impl EventMetadata {
    pub fn new(
        actor_id: Option<String>,
        correlation_id: Option<String>,
        causation_id: Option<String>,
    ) -> Self {
        Self {
            actor_id,
            correlation_id,
            causation_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventStream {
    pub id: String,
    pub aggregate_type: String,
    pub current_version: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl EventStream {
    pub fn new(id: String, aggregate_type: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            aggregate_type,
            current_version: 0,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredEvent {
    pub stream_id: String,
    pub sequence: u64,
    pub event_type: String,
    pub version: i64,
    pub payload: serde_json::Value,
    pub metadata: EventMetadata,
    pub occurred_at: DateTime<Utc>,
    pub stored_at: DateTime<Utc>,
}

impl StoredEvent {
    pub fn new(
        stream_id: String,
        sequence: u64,
        event_type: String,
        version: i64,
        payload: serde_json::Value,
        metadata: EventMetadata,
    ) -> Self {
        let now = Utc::now();
        Self {
            stream_id,
            sequence,
            event_type,
            version,
            payload,
            metadata,
            occurred_at: now,
            stored_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub id: String,
    pub stream_id: String,
    pub snapshot_version: i64,
    pub aggregate_type: String,
    pub state: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

impl Snapshot {
    pub fn new(
        id: String,
        stream_id: String,
        snapshot_version: i64,
        aggregate_type: String,
        state: serde_json::Value,
    ) -> Self {
        Self {
            id,
            stream_id,
            snapshot_version,
            aggregate_type,
            state,
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventData {
    pub event_type: String,
    pub payload: serde_json::Value,
    pub metadata: EventMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationInfo {
    pub total_count: u64,
    pub page: u32,
    pub page_size: u32,
    pub has_next: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_metadata_new() {
        let meta = EventMetadata::new(
            Some("user-001".to_string()),
            Some("corr-001".to_string()),
            None,
        );
        assert_eq!(meta.actor_id.as_deref(), Some("user-001"));
        assert_eq!(meta.correlation_id.as_deref(), Some("corr-001"));
        assert!(meta.causation_id.is_none());
    }

    #[test]
    fn event_stream_new() {
        let stream = EventStream::new("order-001".to_string(), "Order".to_string());
        assert_eq!(stream.id, "order-001");
        assert_eq!(stream.aggregate_type, "Order");
        assert_eq!(stream.current_version, 0);
        assert!(stream.created_at <= Utc::now());
    }

    #[test]
    fn stored_event_new() {
        let meta = EventMetadata::new(Some("user-001".to_string()), None, None);
        let event = StoredEvent::new(
            "order-001".to_string(),
            1,
            "OrderPlaced".to_string(),
            1,
            serde_json::json!({"order_id": "o-1"}),
            meta,
        );
        assert_eq!(event.stream_id, "order-001");
        assert_eq!(event.sequence, 1);
        assert_eq!(event.event_type, "OrderPlaced");
        assert_eq!(event.version, 1);
        assert_eq!(event.payload["order_id"], "o-1");
    }

    #[test]
    fn snapshot_new() {
        let snap = Snapshot::new(
            "snap-001".to_string(),
            "order-001".to_string(),
            5,
            "Order".to_string(),
            serde_json::json!({"status": "completed"}),
        );
        assert_eq!(snap.id, "snap-001");
        assert_eq!(snap.stream_id, "order-001");
        assert_eq!(snap.snapshot_version, 5);
        assert_eq!(snap.aggregate_type, "Order");
        assert_eq!(snap.state["status"], "completed");
    }

    #[test]
    fn event_data_serialization() {
        let data = EventData {
            event_type: "OrderPlaced".to_string(),
            payload: serde_json::json!({"total": 3000}),
            metadata: EventMetadata::new(Some("user-001".to_string()), None, None),
        };
        let json = serde_json::to_string(&data).unwrap();
        let parsed: EventData = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.event_type, "OrderPlaced");
        assert_eq!(parsed.payload["total"], 3000);
    }

    #[test]
    fn pagination_info() {
        let page = PaginationInfo {
            total_count: 100,
            page: 2,
            page_size: 50,
            has_next: false,
        };
        assert_eq!(page.total_count, 100);
        assert_eq!(page.page, 2);
        assert_eq!(page.page_size, 50);
        assert!(!page.has_next);
    }
}
