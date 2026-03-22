// プロジェクトタイプエンティティ。
// ソフトウェア開発・マーケティング等のテンプレート定義を表現する。
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// プロジェクトタイプ（会計の MasterCategory に相当）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectType {
    pub id: Uuid,
    pub code: String,
    pub display_name: String,
    pub description: Option<String>,
    pub default_workflow: Option<serde_json::Value>,
    pub is_active: bool,
    pub sort_order: i32,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// プロジェクトタイプ作成 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProjectType {
    pub code: String,
    pub display_name: String,
    pub description: Option<String>,
    pub default_workflow: Option<serde_json::Value>,
    pub is_active: Option<bool>,
    pub sort_order: Option<i32>,
}

/// プロジェクトタイプ更新 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProjectType {
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub default_workflow: Option<serde_json::Value>,
    pub is_active: Option<bool>,
    pub sort_order: Option<i32>,
}

/// プロジェクトタイプ一覧フィルタ
#[derive(Debug, Clone, Default)]
pub struct ProjectTypeFilter {
    pub active_only: bool,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
