use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasterItemVersion {
    pub id: Uuid,
    pub item_id: Uuid,
    pub version_number: i32,
    pub before_data: Option<serde_json::Value>,
    pub after_data: Option<serde_json::Value>,
    pub changed_by: String,
    pub change_reason: Option<String>,
    pub created_at: DateTime<Utc>,
}
