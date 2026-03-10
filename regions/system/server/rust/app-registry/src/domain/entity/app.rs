use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// App はアプリケーションレジストリに登録されたアプリを表すドメインエンティティ。
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct App {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub icon_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
