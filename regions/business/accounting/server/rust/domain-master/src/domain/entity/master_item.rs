use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasterItem {
    pub id: Uuid,
    pub category_id: Uuid,
    pub code: String,
    pub display_name: String,
    pub description: Option<String>,
    pub attributes: Option<serde_json::Value>,
    pub parent_item_id: Option<Uuid>,
    pub effective_from: Option<DateTime<Utc>>,
    pub effective_until: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub sort_order: i32,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMasterItem {
    pub code: String,
    pub display_name: String,
    pub description: Option<String>,
    pub attributes: Option<serde_json::Value>,
    pub parent_item_id: Option<Uuid>,
    pub effective_from: Option<DateTime<Utc>>,
    pub effective_until: Option<DateTime<Utc>>,
    pub is_active: Option<bool>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMasterItem {
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub attributes: Option<serde_json::Value>,
    pub parent_item_id: Option<Uuid>,
    pub effective_from: Option<DateTime<Utc>>,
    pub effective_until: Option<DateTime<Utc>>,
    pub is_active: Option<bool>,
    pub sort_order: Option<i32>,
}
