use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnDefinition {
    pub id: Uuid,
    pub table_id: Uuid,
    pub column_name: String,
    pub display_name: String,
    pub data_type: String,
    pub is_primary_key: bool,
    pub is_nullable: bool,
    pub is_unique: bool,
    pub default_value: Option<String>,
    pub max_length: Option<i32>,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub regex_pattern: Option<String>,
    pub display_order: i32,
    pub is_searchable: bool,
    pub is_sortable: bool,
    pub is_filterable: bool,
    pub is_visible_in_list: bool,
    pub is_visible_in_form: bool,
    pub is_readonly: bool,
    pub input_type: String,
    pub select_options: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateColumnDefinition {
    pub column_name: String,
    pub display_name: String,
    pub data_type: String,
    pub is_primary_key: Option<bool>,
    pub is_nullable: Option<bool>,
    pub is_unique: Option<bool>,
    pub default_value: Option<String>,
    pub max_length: Option<i32>,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub regex_pattern: Option<String>,
    pub display_order: Option<i32>,
    pub is_searchable: Option<bool>,
    pub is_sortable: Option<bool>,
    pub is_filterable: Option<bool>,
    pub is_visible_in_list: Option<bool>,
    pub is_visible_in_form: Option<bool>,
    pub is_readonly: Option<bool>,
    pub input_type: Option<String>,
    pub select_options: Option<serde_json::Value>,
}
