use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationChannel {
    pub id: String,
    pub name: String,
    pub channel_type: String,
    pub config: serde_json::Value,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl NotificationChannel {
    pub fn new(
        name: String,
        channel_type: String,
        config: serde_json::Value,
        enabled: bool,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: format!("ch_{}", uuid::Uuid::new_v4().simple()),
            name,
            channel_type,
            config,
            enabled,
            created_at: now,
            updated_at: now,
        }
    }
}
