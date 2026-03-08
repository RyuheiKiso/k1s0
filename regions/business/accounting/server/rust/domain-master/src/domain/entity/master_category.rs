use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasterCategory {
    pub id: Uuid,
    pub code: String,
    pub display_name: String,
    pub description: Option<String>,
    pub validation_schema: Option<serde_json::Value>,
    pub is_active: bool,
    pub sort_order: i32,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMasterCategory {
    pub code: String,
    pub display_name: String,
    pub description: Option<String>,
    pub validation_schema: Option<serde_json::Value>,
    pub is_active: Option<bool>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMasterCategory {
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub validation_schema: Option<serde_json::Value>,
    pub is_active: Option<bool>,
    pub sort_order: Option<i32>,
}
