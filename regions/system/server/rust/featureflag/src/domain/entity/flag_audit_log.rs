use chrono::{DateTime, Utc};
use uuid::Uuid;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FlagAuditLog {
    pub id: Uuid,
    pub flag_id: Uuid,
    pub flag_key: String,
    pub action: String,
    pub before_json: Option<serde_json::Value>,
    pub after_json: Option<serde_json::Value>,
    pub changed_by: String,
    pub trace_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl FlagAuditLog {
    pub fn new(
        flag_id: Uuid,
        flag_key: String,
        action: String,
        before_json: Option<serde_json::Value>,
        after_json: Option<serde_json::Value>,
        changed_by: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            flag_id,
            flag_key,
            action,
            before_json,
            after_json,
            changed_by,
            trace_id: None,
            created_at: Utc::now(),
        }
    }
}
