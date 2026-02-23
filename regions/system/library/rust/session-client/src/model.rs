use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub revoked: bool,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct CreateSessionRequest {
    pub user_id: String,
    pub ttl_seconds: i64,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct RefreshSessionRequest {
    pub id: String,
    pub ttl_seconds: i64,
}
