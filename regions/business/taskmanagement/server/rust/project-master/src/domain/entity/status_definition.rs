// ステータス定義エンティティ。
// Open/In Progress/Review/Done 等の共通ステータス定義を表現する（会計の MasterItem に相当）。
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// ステータス定義（会計の MasterItem に相当）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusDefinition {
    pub id: Uuid,
    pub project_type_id: Uuid,
    pub code: String,
    pub display_name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub allowed_transitions: Option<serde_json::Value>,
    pub is_initial: bool,
    pub is_terminal: bool,
    pub sort_order: i32,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// ステータス定義作成 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateStatusDefinition {
    pub project_type_id: Uuid,
    pub code: String,
    pub display_name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub allowed_transitions: Option<serde_json::Value>,
    pub is_initial: Option<bool>,
    pub is_terminal: Option<bool>,
    pub sort_order: Option<i32>,
}

/// ステータス定義更新 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateStatusDefinition {
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub color: Option<String>,
    pub allowed_transitions: Option<serde_json::Value>,
    pub is_initial: Option<bool>,
    pub is_terminal: Option<bool>,
    pub sort_order: Option<i32>,
    pub change_reason: Option<String>,
}

/// ステータス定義一覧フィルタ
#[derive(Debug, Clone, Default)]
pub struct StatusDefinitionFilter {
    pub project_type_id: Option<Uuid>,
    pub active_only: bool,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
