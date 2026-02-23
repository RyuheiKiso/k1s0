use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationLog {
    pub id: Uuid,
    pub channel_id: Uuid,
    pub template_id: Option<Uuid>,
    pub recipient: String,
    pub subject: Option<String>,
    pub body: String,
    pub status: String,
    pub error_message: Option<String>,
    pub sent_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl NotificationLog {
    pub fn new(channel_id: Uuid, recipient: String, subject: Option<String>, body: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            channel_id,
            template_id: None,
            recipient,
            subject,
            body,
            status: "pending".to_string(),
            error_message: None,
            sent_at: None,
            created_at: Utc::now(),
        }
    }
}
