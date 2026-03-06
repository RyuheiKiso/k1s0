use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationLog {
    pub id: String,
    pub channel_id: String,
    pub template_id: Option<String>,
    pub recipient: String,
    pub subject: Option<String>,
    pub body: String,
    pub status: String,
    pub retry_count: u32,
    pub error_message: Option<String>,
    pub sent_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl NotificationLog {
    pub fn new(
        channel_id: String,
        recipient: String,
        subject: Option<String>,
        body: String,
    ) -> Self {
        Self {
            id: format!("notif_{}", uuid::Uuid::new_v4().simple()),
            channel_id,
            template_id: None,
            recipient,
            subject,
            body,
            status: "pending".to_string(),
            retry_count: 0,
            error_message: None,
            sent_at: None,
            created_at: Utc::now(),
        }
    }
}
