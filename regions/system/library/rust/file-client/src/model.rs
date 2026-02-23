use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub path: String,
    pub size_bytes: i64,
    pub content_type: String,
    pub etag: String,
    pub last_modified: DateTime<Utc>,
    pub tags: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresignedUrl {
    pub url: String,
    pub method: String,
    pub expires_at: DateTime<Utc>,
    pub headers: HashMap<String, String>,
}
