use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NotificationChannel {
    Email,
    Sms,
    Push,
    Slack,
    Webhook,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationRequest {
    pub id: Uuid,
    pub channel: NotificationChannel,
    pub recipient: String,
    pub subject: Option<String>,
    pub body: String,
    pub metadata: Option<serde_json::Value>,
}

impl NotificationRequest {
    pub fn new(
        channel: NotificationChannel,
        recipient: impl Into<String>,
        body: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            channel,
            recipient: recipient.into(),
            subject: None,
            body: body.into(),
            metadata: None,
        }
    }

    pub fn with_subject(mut self, subject: impl Into<String>) -> Self {
        self.subject = Some(subject.into());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationResponse {
    pub id: Uuid,
    pub status: String,
    pub message_id: Option<String>,
}
