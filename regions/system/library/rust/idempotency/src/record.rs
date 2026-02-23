use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IdempotencyStatus {
    Pending,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdempotencyRecord {
    pub key: String,
    pub status: IdempotencyStatus,
    pub request_hash: Option<String>,
    pub response_body: Option<String>,
    pub response_status: Option<u16>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl IdempotencyRecord {
    pub fn new(key: String, ttl_secs: Option<i64>) -> Self {
        let now = Utc::now();
        Self {
            key,
            status: IdempotencyStatus::Pending,
            request_hash: None,
            response_body: None,
            response_status: None,
            created_at: now,
            expires_at: ttl_secs.map(|s| now + chrono::Duration::seconds(s)),
            completed_at: None,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.expires_at.map_or(false, |exp| exp <= Utc::now())
    }

    pub fn complete(mut self, response_body: Option<String>, response_status: Option<u16>) -> Self {
        self.status = IdempotencyStatus::Completed;
        self.response_body = response_body;
        self.response_status = response_status;
        self.completed_at = Some(Utc::now());
        self
    }

    pub fn fail(mut self, error: String) -> Self {
        self.status = IdempotencyStatus::Failed;
        self.response_body = Some(error);
        self.completed_at = Some(Utc::now());
        self
    }
}
