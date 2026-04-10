use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// H-012 監査対応: `tenant_id` フィールドを追加してマルチテナント分離を実現する
/// C-005 監査対応: config フィールドは DB 保存時に AES-256-GCM で暗号化される
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationChannel {
    pub id: String,
    pub name: String,
    pub channel_type: String,
    /// チャンネル設定（SMTP ホスト、API キー等の機密情報を含む可能性がある）
    /// DB 保存時は `encrypted_config` として AES-256-GCM で暗号化される
    pub config: serde_json::Value,
    /// H-012 監査対応: テナント識別子。システム共通チャンネルは "system"
    pub tenant_id: String,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl NotificationChannel {
    /// テナント ID を指定してチャンネルを作成する
    #[must_use]
    pub fn new(
        name: String,
        channel_type: String,
        config: serde_json::Value,
        tenant_id: String,
        enabled: bool,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: format!("ch_{}", uuid::Uuid::new_v4().simple()),
            name,
            channel_type,
            config,
            tenant_id,
            enabled,
            created_at: now,
            updated_at: now,
        }
    }

    /// `システムチャンネル（tenant_id` = "system"）として作成する（テスト・将来用）
    #[allow(dead_code)]
    #[must_use]
    pub fn new_system(
        name: String,
        channel_type: String,
        config: serde_json::Value,
        enabled: bool,
    ) -> Self {
        Self::new(name, channel_type, config, "system".to_string(), enabled)
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
            "system".to_string(),
            true,
        );
        assert!(ch.id.starts_with("ch_"));
        assert_eq!(ch.name, "email-channel");
        assert_eq!(ch.channel_type, "email");
        assert_eq!(ch.tenant_id, "system");
        assert!(ch.enabled);
    }

    /// 無効チャンネルは enabled=false で生成される
    #[test]
    fn new_disabled_channel() {
        let ch = NotificationChannel::new(
            "sms".to_string(),
            "sms".to_string(),
            serde_json::json!({}),
            "tenant-abc".to_string(),
            false,
        );
        assert!(!ch.enabled);
        assert_eq!(ch.tenant_id, "tenant-abc");
    }

    /// 複数のチャンネルは異なる ID を持つ
    #[test]
    fn unique_ids() {
        let ch1 = NotificationChannel::new(
            "a".to_string(),
            "email".to_string(),
            serde_json::json!({}),
            "system".to_string(),
            true,
        );
        let ch2 = NotificationChannel::new(
            "b".to_string(),
            "email".to_string(),
            serde_json::json!({}),
            "system".to_string(),
            true,
        );
        assert_ne!(ch1.id, ch2.id);
    }

    /// new_system が tenant_id = "system" でチャンネルを作成する
    #[test]
    fn new_system_channel() {
        let ch = NotificationChannel::new_system(
            "webhook".to_string(),
            "webhook".to_string(),
            serde_json::json!({"url": "https://hooks.example.com"}),
            true,
        );
        assert_eq!(ch.tenant_id, "system");
    }
}
