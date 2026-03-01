use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleCondition {
    pub id: Uuid,
    pub rule_id: Uuid,
    pub condition_order: i32,
    pub left_table_id: Uuid,
    pub left_column: String,
    pub operator: String,
    pub right_table_id: Option<Uuid>,
    pub right_column: Option<String>,
    pub right_value: Option<String>,
    pub logical_connector: Option<String>,
    pub created_at: DateTime<Utc>,
}
