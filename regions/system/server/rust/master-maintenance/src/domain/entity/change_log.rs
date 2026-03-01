use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeLog {
    pub id: Uuid,
    pub target_table: String,
    pub target_record_id: String,
    pub operation: String,
    pub before_data: Option<serde_json::Value>,
    pub after_data: Option<serde_json::Value>,
    pub changed_columns: Option<Vec<String>>,
    pub changed_by: String,
    pub change_reason: Option<String>,
    pub trace_id: Option<String>,
    pub created_at: DateTime<Utc>,
}
