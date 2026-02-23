use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitStatus {
    pub allowed: bool,
    pub remaining: u32,
    pub reset_at: DateTime<Utc>,
    pub retry_after_secs: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitResult {
    pub remaining: u32,
    pub reset_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitPolicy {
    pub key: String,
    pub limit: u32,
    pub window_secs: u64,
    pub algorithm: String,
}
