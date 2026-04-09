use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// テーブル定義エンティティは複数の権限制御用 boolean フィールドを持つ（DBスキーマとの整合性上必要）
#[allow(clippy::struct_excessive_bools)]
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
    #[serde(default)]
    pub read_roles: Vec<String>,
    #[serde(default)]
    pub write_roles: Vec<String>,
    #[serde(default)]
    pub admin_roles: Vec<String>,
    pub sort_order: i32,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub domain_scope: Option<String>,
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
    pub read_roles: Option<Vec<String>>,
    pub write_roles: Option<Vec<String>>,
    pub admin_roles: Option<Vec<String>>,
    pub sort_order: Option<i32>,
    pub domain_scope: Option<String>,
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
    pub read_roles: Option<Vec<String>>,
    pub write_roles: Option<Vec<String>>,
    pub admin_roles: Option<Vec<String>>,
    pub sort_order: Option<i32>,
}
