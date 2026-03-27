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

#[cfg(test)]
mod tests {
    use super::*;

    /// NotificationChannel::new が ch_ プレフィックス付きの ID を生成する
    #[test]
    fn new_channel_id_has_prefix() {
        let ch = NotificationChannel::new(
            "email-channel".to_string(),
            "email".to_string(),
            serde_json::json!({"smtp": "localhost"}),
            true,
        );
        assert!(ch.id.starts_with("ch_"));
        assert_eq!(ch.name, "email-channel");
        assert_eq!(ch.channel_type, "email");
        assert!(ch.enabled);
    }

    /// 無効チャンネルは enabled=false で生成される
    #[test]
    fn new_disabled_channel() {
        let ch = NotificationChannel::new(
            "sms".to_string(),
            "sms".to_string(),
            serde_json::json!({}),
            false,
        );
        assert!(!ch.enabled);
    }

    /// 複数のチャンネルは異なる ID を持つ
    #[test]
    fn unique_ids() {
        let ch1 = NotificationChannel::new(
            "a".to_string(),
            "email".to_string(),
            serde_json::json!({}),
            true,
        );
        let ch2 = NotificationChannel::new(
            "b".to_string(),
            "email".to_string(),
            serde_json::json!({}),
            true,
        );
        assert_ne!(ch1.id, ch2.id);
    }
}
