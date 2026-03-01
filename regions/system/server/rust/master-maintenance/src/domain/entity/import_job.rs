use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportJob {
    pub id: Uuid,
    pub table_id: Uuid,
    pub file_name: String,
    pub status: String,
    pub total_rows: i32,
    pub processed_rows: i32,
    pub error_rows: i32,
    pub error_details: Option<serde_json::Value>,
    pub started_by: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}
