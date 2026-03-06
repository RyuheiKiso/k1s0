use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationTemplate {
    pub id: String,
    pub name: String,
    pub channel_type: String,
    pub subject_template: Option<String>,
    pub body_template: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl NotificationTemplate {
    pub fn new(
        name: String,
        channel_type: String,
        subject_template: Option<String>,
        body_template: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: format!("tpl_{}", uuid::Uuid::new_v4().simple()),
            name,
            channel_type,
            subject_template,
            body_template,
            created_at: now,
            updated_at: now,
        }
    }
}
