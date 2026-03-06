use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct EventRecord {
    pub id: Uuid,
    pub correlation_id: String,
    pub event_type: String,
    pub source: String,
    pub domain: String,
    pub trace_id: String,
    pub timestamp: DateTime<Utc>,
    pub flow_id: Option<Uuid>,
    pub flow_step_index: Option<i32>,
    pub status: String,
    pub received_at: DateTime<Utc>,
}

impl EventRecord {
    pub fn new(
        correlation_id: String,
        event_type: String,
        source: String,
        domain: String,
        trace_id: String,
        timestamp: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            correlation_id,
            event_type,
            source,
            domain,
            trace_id,
            timestamp,
            flow_id: None,
            flow_step_index: None,
            status: "normal".to_string(),
            received_at: Utc::now(),
        }
    }
}
