use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultipartPart {
    pub part_number: i32,
    pub etag: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultipartUploadSession {
    pub upload_id: String,
    pub bucket: String,
    pub key: String,
    pub started_at: DateTime<Utc>,
    pub parts: Vec<MultipartPart>,
}

impl MultipartUploadSession {
    pub fn new(
        upload_id: impl Into<String>,
        bucket: impl Into<String>,
        key: impl Into<String>,
    ) -> Self {
        Self {
            upload_id: upload_id.into(),
            bucket: bucket.into(),
            key: key.into(),
            started_at: Utc::now(),
            parts: Vec::new(),
        }
    }

    pub fn add_part(&mut self, part_number: i32, etag: impl Into<String>) {
        self.parts.push(MultipartPart {
            part_number,
            etag: etag.into(),
        });
    }
}
