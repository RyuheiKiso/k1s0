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
    #[must_use]
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

#[cfg(test)]
mod tests {
    use super::*;

    /// NotificationLog::new が notif_ プレフィックスの ID と pending ステータスで生成される
    #[test]
    fn new_pending_status() {
        let log = NotificationLog::new(
            "ch_abc".to_string(),
            "user@example.com".to_string(),
            Some("Alert".to_string()),
            "Server is down".to_string(),
        );
        assert!(log.id.starts_with("notif_"));
        assert_eq!(log.status, "pending");
        assert_eq!(log.retry_count, 0);
        assert!(log.sent_at.is_none());
        assert!(log.error_message.is_none());
        assert!(log.template_id.is_none());
    }

    /// subject が None の場合も正常に生成される
    #[test]
    fn new_without_subject() {
        let log = NotificationLog::new(
            "ch_slack".to_string(),
            "#alerts".to_string(),
            None,
            "Deployment completed".to_string(),
        );
        assert!(log.subject.is_none());
        assert_eq!(log.body, "Deployment completed");
    }
}
