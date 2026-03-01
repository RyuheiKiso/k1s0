use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsistencyRule {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub rule_type: String,
    pub severity: String,
    pub is_active: bool,
    pub source_table_id: Uuid,
    pub evaluation_timing: String,
    pub error_message_template: String,
    pub zen_rule_json: Option<serde_json::Value>,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateConsistencyRule {
    pub name: String,
    pub description: Option<String>,
    pub rule_type: String,
    pub severity: Option<String>,
    pub source_table: String,
    pub evaluation_timing: Option<String>,
    pub error_message_template: String,
    pub zen_rule_json: Option<serde_json::Value>,
    pub conditions: Option<Vec<CreateRuleConditionInput>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRuleConditionInput {
    pub condition_order: i32,
    pub left_column: String,
    pub operator: String,
    pub right_table: Option<String>,
    pub right_column: Option<String>,
    pub right_value: Option<String>,
    pub logical_connector: Option<String>,
}
