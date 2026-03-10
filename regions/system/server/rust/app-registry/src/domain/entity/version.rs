use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use super::platform::Platform;

/// AppVersion はアプリの特定バージョン・プラットフォーム向けのリリースを表すドメインエンティティ。
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AppVersion {
    pub id: Uuid,
    pub app_id: String,
    pub version: String,
    pub platform: Platform,
    pub arch: String,
    pub size_bytes: Option<i64>,
    pub checksum_sha256: String,
    pub s3_key: String,
    pub release_notes: Option<String>,
    pub mandatory: bool,
    pub published_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}
