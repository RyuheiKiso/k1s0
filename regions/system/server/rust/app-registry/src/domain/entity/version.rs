use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use super::platform::Platform;

/// `AppVersion` はアプリの特定バージョン・プラットフォーム向けのリリースを表すドメインエンティティ。
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AppVersion {
    pub id: Uuid,
    pub app_id: String,
    pub version: String,
    pub platform: Platform,
    pub arch: String,
    pub size_bytes: Option<i64>,
    pub checksum_sha256: String,
    pub storage_key: String,
    pub release_notes: Option<String>,
    pub mandatory: bool,
    /// STATIC-CRITICAL-002: Cosign 署名文字列（base64 エンコード）。
    /// 本番環境では必須。開発環境は None 許可（検証スキップ）。
    pub cosign_signature: Option<String>,
    pub published_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}
