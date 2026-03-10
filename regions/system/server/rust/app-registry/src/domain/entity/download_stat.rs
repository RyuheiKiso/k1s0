use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// DownloadStat はアプリダウンロードの統計情報を表すドメインエンティティ。
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DownloadStat {
    pub id: Uuid,
    pub app_id: String,
    pub version: String,
    pub platform: String,
    pub user_id: String,
    pub downloaded_at: DateTime<Utc>,
}
