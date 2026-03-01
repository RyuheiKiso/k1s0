use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    pub id: Uuid,
    pub table_id: Uuid,
    pub config_type: String,
    pub config_json: serde_json::Value,
    pub is_default: bool,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDisplayConfig {
    pub config_type: String,
    pub config_json: serde_json::Value,
    pub is_default: Option<bool>,
}
