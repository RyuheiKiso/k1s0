use std::sync::Arc;

use crate::domain::entity::notification_channel::NotificationChannel;
use crate::domain::repository::NotificationChannelRepository;
use crate::domain::service::NotificationDomainService;

/// H-012 監査対応: tenant_id フィールドを追加してマルチテナント分離を実現する
#[derive(Debug, Clone)]
pub struct CreateChannelInput {
    pub name: String,
    pub channel_type: String,
    pub config: serde_json::Value,
    /// テナント識別子。システム共通チャンネルは "system" を指定する
    pub tenant_id: String,
    pub enabled: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum CreateChannelError {
    #[error("validation error: {0}")]
    Validation(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct CreateChannelUseCase {
    repo: Arc<dyn NotificationChannelRepository>,
}

impl CreateChannelUseCase {
    pub fn new(repo: Arc<dyn NotificationChannelRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        input: &CreateChannelInput,
    ) -> Result<NotificationChannel, CreateChannelError> {
        NotificationDomainService::validate_channel_type(&input.channel_type)
            .map_err(CreateChannelError::Validation)?;

        // CRIT-02 / HIGH-03 SSRF対策: webhook チャンネル作成時に URL を検証する
        // config.url フィールドが存在する場合のみ検証を行い、不正な URL は作成を拒否する
        // HIGH-03 DNS リバインド攻撃対策のため async 関数として DNS 解決も実施する
        if input.channel_type == "webhook" {
            if let Some(url) = input.config.get("url").and_then(|v| v.as_str()) {
                NotificationDomainService::validate_webhook_url(url)
                    .await
                    .map_err(CreateChannelError::Validation)?;
            } else {
                return Err(CreateChannelError::Validation(
                    "webhookチャンネルには config.url が必須です".to_string(),
                ));
            }
        }

        // H-012: tenant_id を含めてチャンネルを作成する
        let channel = NotificationChannel::new(
            input.name.clone(),
            input.channel_type.clone(),
            input.config.clone(),
            input.tenant_id.clone(),
            input.enabled,
        );

        self.repo
            .create(&channel)
            .await
            .map_err(|e| CreateChannelError::Internal(e.to_string()))?;

        Ok(channel)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::repository::notification_channel_repository::MockNotificationChannelRepository;

    #[tokio::test]
    async fn success() {
        let mut mock = MockNotificationChannelRepository::new();
        mock.expect_create().returning(|_| Ok(()));

        let uc = CreateChannelUseCase::new(Arc::new(mock));
        let input = CreateChannelInput {
            name: "email-channel".to_string(),
            channel_type: "email".to_string(),
            config: serde_json::json!({"smtp_host": "localhost"}),
            tenant_id: "system".to_string(),
            enabled: true,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let channel = result.unwrap();
        assert_eq!(channel.name, "email-channel");
        assert_eq!(channel.channel_type, "email");
        assert_eq!(channel.tenant_id, "system");
        assert!(channel.enabled);
    }

    #[tokio::test]
    async fn internal_error() {
        let mut mock = MockNotificationChannelRepository::new();
        mock.expect_create()
            .returning(|_| Err(anyhow::anyhow!("db error")));

        let uc = CreateChannelUseCase::new(Arc::new(mock));
        let input = CreateChannelInput {
            name: "sms-channel".to_string(),
            channel_type: "sms".to_string(),
            config: serde_json::json!({}),
            tenant_id: "system".to_string(),
            enabled: true,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            CreateChannelError::Internal(msg) => assert!(msg.contains("db error")),
            CreateChannelError::Validation(msg) => {
                panic!("unexpected validation error: {msg}")
            }
        }
    }

    /// CRIT-02: 正当な外部 HTTPS URL を持つ webhook チャンネルは作成できることを確認する
    #[tokio::test]
    async fn webhook_valid_url_succeeds() {
        let mut mock = MockNotificationChannelRepository::new();
        mock.expect_create().returning(|_| Ok(()));

        let uc = CreateChannelUseCase::new(Arc::new(mock));
        let input = CreateChannelInput {
            name: "webhook-channel".to_string(),
            channel_type: "webhook".to_string(),
            config: serde_json::json!({"url": "https://example.com/hook"}),
            tenant_id: "system".to_string(),
            enabled: true,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());
    }

    /// CRIT-02: プライベートIPアドレスへの webhook チャンネル作成は拒否されることを確認する
    #[tokio::test]
    async fn webhook_private_ip_is_rejected() {
        let mock = MockNotificationChannelRepository::new();
        let uc = CreateChannelUseCase::new(Arc::new(mock));
        let input = CreateChannelInput {
            name: "webhook-channel".to_string(),
            channel_type: "webhook".to_string(),
            config: serde_json::json!({"url": "http://192.168.1.100/hook"}),
            tenant_id: "system".to_string(),
            enabled: true,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            CreateChannelError::Validation(msg) => {
                assert!(msg.contains("プライベートIPアドレス"), "msg: {msg}");
            }
            CreateChannelError::Internal(msg) => panic!("unexpected internal error: {msg}"),
        }
    }

    /// CRIT-02: クラスタ内部ホスト名への webhook チャンネル作成は拒否されることを確認する
    #[tokio::test]
    async fn webhook_cluster_internal_hostname_is_rejected() {
        let mock = MockNotificationChannelRepository::new();
        let uc = CreateChannelUseCase::new(Arc::new(mock));
        let input = CreateChannelInput {
            name: "webhook-channel".to_string(),
            channel_type: "webhook".to_string(),
            config: serde_json::json!({"url": "http://my-service.default.svc.cluster.local/hook"}),
            tenant_id: "system".to_string(),
            enabled: true,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            CreateChannelError::Validation(msg) => {
                assert!(msg.contains("クラスタ内部"), "msg: {msg}");
            }
            CreateChannelError::Internal(msg) => panic!("unexpected internal error: {msg}"),
        }
    }

    /// CRIT-02: config.url がない webhook チャンネル作成は拒否されることを確認する
    #[tokio::test]
    async fn webhook_missing_url_is_rejected() {
        let mock = MockNotificationChannelRepository::new();
        let uc = CreateChannelUseCase::new(Arc::new(mock));
        let input = CreateChannelInput {
            name: "webhook-channel".to_string(),
            channel_type: "webhook".to_string(),
            config: serde_json::json!({}),
            tenant_id: "system".to_string(),
            enabled: true,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            CreateChannelError::Validation(msg) => {
                assert!(msg.contains("config.url"), "msg: {msg}");
            }
            CreateChannelError::Internal(msg) => panic!("unexpected internal error: {msg}"),
        }
    }

    /// CRIT-02: localhost への webhook チャンネル作成は拒否されることを確認する
    #[tokio::test]
    async fn webhook_localhost_is_rejected() {
        let mock = MockNotificationChannelRepository::new();
        let uc = CreateChannelUseCase::new(Arc::new(mock));
        let input = CreateChannelInput {
            name: "webhook-channel".to_string(),
            channel_type: "webhook".to_string(),
            config: serde_json::json!({"url": "http://localhost/hook"}),
            tenant_id: "system".to_string(),
            enabled: true,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            CreateChannelError::Validation(msg) => {
                assert!(msg.contains("クラスタ内部"), "msg: {msg}");
            }
            CreateChannelError::Internal(msg) => panic!("unexpected internal error: {msg}"),
        }
    }
}
