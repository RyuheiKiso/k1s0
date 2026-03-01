use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableDefinition {
    pub id: Uuid,
    pub name: String,
    pub schema_name: String,
    pub database_name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub is_active: bool,
    pub allow_create: bool,
    pub allow_update: bool,
    pub allow_delete: bool,
    pub sort_order: i32,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTableDefinition {
    pub name: String,
    pub schema_name: String,
    pub database_name: Option<String>,
    pub display_name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub allow_create: Option<bool>,
    pub allow_update: Option<bool>,
    pub allow_delete: Option<bool>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTableDefinition {
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub is_active: Option<bool>,
    pub allow_create: Option<bool>,
    pub allow_update: Option<bool>,
    pub allow_delete: Option<bool>,
    pub sort_order: Option<i32>,
}
