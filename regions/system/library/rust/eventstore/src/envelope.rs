use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::StreamId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub event_id: Uuid,
    pub stream_id: String,
    pub version: u64,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub metadata: serde_json::Value,
    pub recorded_at: DateTime<Utc>,
}

impl EventEnvelope {
    pub fn new(
        stream_id: &StreamId,
        version: u64,
        event_type: impl Into<String>,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            stream_id: stream_id.to_string(),
            version,
            event_type: event_type.into(),
            payload,
            metadata: serde_json::Value::Object(serde_json::Map::new()),
            recorded_at: Utc::now(),
        }
    }
}
