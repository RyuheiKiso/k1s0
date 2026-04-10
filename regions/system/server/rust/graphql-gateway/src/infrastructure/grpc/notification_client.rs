use std::collections::HashMap;
use std::time::Duration;

use tonic::transport::Channel;
use tracing::instrument;

use crate::domain::model::{NotificationChannel, NotificationLog, NotificationTemplate};
use crate::infrastructure::config::BackendConfig;

// HIGH-001 監査対応: tonic::include_proto!で展開される生成コードのClippy警告を抑制する
#[allow(
    dead_code,
    clippy::default_trait_access,
    clippy::trivially_copy_pass_by_ref,
    clippy::too_many_lines,
    clippy::doc_markdown,
    clippy::must_use_candidate
)]
pub mod proto {
    pub mod k1s0 {
        pub mod system {
            pub mod common {
                pub mod v1 {
                    tonic::include_proto!("k1s0.system.common.v1");
                }
            }
            pub mod notification {
                pub mod v1 {
                    tonic::include_proto!("k1s0.system.notification.v1");
                }
            }
        }
    }
}

use proto::k1s0::system::notification::v1::notification_service_client::NotificationServiceClient;

pub struct NotificationGrpcClient {
    client: NotificationServiceClient<Channel>,
    /// バックエンドサービスのアドレス。gRPC Health Check Protocol のためのチャネル生成に使用する。
    address: String,
    /// `タイムアウト設定（ミリ秒）。health_check` のチャネル生成にも適用する。
    timeout_ms: u64,
}

impl NotificationGrpcClient {
    /// バックエンド設定からクライアントを生成する。
    /// `connect_lazy()` により起動時の接続確立を不要とし、実際のRPC呼び出し時に接続する。
    pub fn new(cfg: &BackendConfig) -> anyhow::Result<Self> {
        let channel = Channel::from_shared(cfg.address.clone())?
            .timeout(Duration::from_millis(cfg.timeout_ms))
            .connect_lazy();
        Ok(Self {
            client: NotificationServiceClient::new(channel),
            address: cfg.address.clone(),
            timeout_ms: cfg.timeout_ms,
        })
    }

    // H-02 監査対応: proto 生成コードの deprecated フィールドアクセス（後方互換維持のため）
    #[allow(deprecated)]
    fn log_from_proto(
        n: proto::k1s0::system::notification::v1::NotificationLog,
    ) -> NotificationLog {
        NotificationLog {
            id: n.id,
            channel_id: n.channel_id,
            channel_type: n.channel_type,
            template_id: n.template_id.filter(|s| !s.is_empty()),
            recipient: n.recipient,
            subject: n.subject.filter(|s| !s.is_empty()),
            body: n.body,
            status: n.status,
            // LOW-008: 安全な型変換（オーバーフロー防止）
            retry_count: i32::try_from(n.retry_count).unwrap_or(i32::MAX),
            error_message: n.error_message.filter(|s| !s.is_empty()),
            sent_at: n.sent_at.filter(|s| !s.is_empty()),
            created_at: n.created_at,
        }
    }

    // H-02 監査対応: proto 生成コードの deprecated フィールドアクセス（後方互換維持のため）
    #[allow(deprecated)]
    fn channel_from_proto(
        c: proto::k1s0::system::notification::v1::Channel,
    ) -> NotificationChannel {
        NotificationChannel {
            id: c.id,
            name: c.name,
            channel_type: c.channel_type,
            config_json: c.config_json,
            enabled: c.enabled,
            created_at: c.created_at,
            updated_at: c.updated_at,
        }
    }

    // H-02 監査対応: proto 生成コードの deprecated フィールドアクセス（後方互換維持のため）
    #[allow(deprecated)]
    fn template_from_proto(
        t: proto::k1s0::system::notification::v1::Template,
    ) -> NotificationTemplate {
        NotificationTemplate {
            id: t.id,
            name: t.name,
            channel_type: t.channel_type,
            subject_template: t.subject_template.filter(|s| !s.is_empty()),
            body_template: t.body_template,
            created_at: t.created_at,
            updated_at: t.updated_at,
        }
    }

    // ── Notification Log ──

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn get_notification(
        &self,
        notification_id: &str,
    ) -> anyhow::Result<Option<NotificationLog>> {
        let request = tonic::Request::new(
            proto::k1s0::system::notification::v1::GetNotificationRequest {
                notification_id: notification_id.to_owned(),
            },
        );

        match self.client.clone().get_notification(request).await {
            Ok(resp) => {
                // 通知が存在しない場合は None を返す
                let Some(n) = resp.into_inner().notification else {
                    return Ok(None);
                };
                Ok(Some(Self::log_from_proto(n)))
            }
            Err(status) if status.code() == tonic::Code::NotFound => Ok(None),
            Err(e) => Err(anyhow::anyhow!(
                "NotificationService.GetNotification failed: {e}"
            )),
        }
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn list_notifications(
        &self,
        channel_id: Option<&str>,
        status: Option<&str>,
        page: Option<i32>,
        page_size: Option<i32>,
    ) -> anyhow::Result<Vec<NotificationLog>> {
        // ページネーションパラメータを共通Paginationサブメッセージで構成
        let request = tonic::Request::new(
            proto::k1s0::system::notification::v1::ListNotificationsRequest {
                channel_id: channel_id.map(std::borrow::ToOwned::to_owned),
                status: status.map(std::borrow::ToOwned::to_owned),
                pagination: Some(proto::k1s0::system::common::v1::Pagination {
                    page: page.unwrap_or(1),
                    page_size: page_size.unwrap_or(20),
                }),
            },
        );

        let resp = self
            .client
            .clone()
            .list_notifications(request)
            .await
            .map_err(|e| anyhow::anyhow!("NotificationService.ListNotifications failed: {e}"))?
            .into_inner();

        Ok(resp
            .notifications
            .into_iter()
            .map(Self::log_from_proto)
            .collect())
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    // H-02 監査対応: proto 生成コードの deprecated フィールドアクセス（後方互換維持のため）
    #[allow(deprecated)]
    pub async fn send_notification(
        &self,
        channel_id: &str,
        template_id: Option<&str>,
        template_variables: &HashMap<String, String>,
        recipient: &str,
        subject: Option<&str>,
        body: Option<&str>,
    ) -> anyhow::Result<(String, String, String)> {
        let request = tonic::Request::new(
            proto::k1s0::system::notification::v1::SendNotificationRequest {
                channel_id: channel_id.to_owned(),
                template_id: template_id.map(std::borrow::ToOwned::to_owned),
                template_variables: template_variables.clone(),
                recipient: recipient.to_owned(),
                subject: subject.map(std::borrow::ToOwned::to_owned),
                body: body.map(std::borrow::ToOwned::to_owned),
            },
        );

        let resp = self
            .client
            .clone()
            .send_notification(request)
            .await
            .map_err(|e| anyhow::anyhow!("NotificationService.SendNotification failed: {e}"))?
            .into_inner();

        Ok((resp.notification_id, resp.status, resp.created_at))
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn retry_notification(
        &self,
        notification_id: &str,
    ) -> anyhow::Result<NotificationLog> {
        let request = tonic::Request::new(
            proto::k1s0::system::notification::v1::RetryNotificationRequest {
                notification_id: notification_id.to_owned(),
            },
        );

        let n = self
            .client
            .clone()
            .retry_notification(request)
            .await
            .map_err(|e| anyhow::anyhow!("NotificationService.RetryNotification failed: {e}"))?
            .into_inner()
            .notification
            .ok_or_else(|| anyhow::anyhow!("empty notification in retry response"))?;

        Ok(Self::log_from_proto(n))
    }

    // ── Channel ──

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn get_channel(&self, id: &str) -> anyhow::Result<Option<NotificationChannel>> {
        let request =
            tonic::Request::new(proto::k1s0::system::notification::v1::GetChannelRequest {
                id: id.to_owned(),
            });

        match self.client.clone().get_channel(request).await {
            Ok(resp) => {
                // チャンネルが存在しない場合は None を返す
                let Some(c) = resp.into_inner().channel else {
                    return Ok(None);
                };
                Ok(Some(Self::channel_from_proto(c)))
            }
            Err(status) if status.code() == tonic::Code::NotFound => Ok(None),
            Err(e) => Err(anyhow::anyhow!(
                "NotificationService.GetChannel failed: {e}"
            )),
        }
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn list_channels(
        &self,
        channel_type: Option<&str>,
        enabled_only: bool,
        page: Option<i32>,
        page_size: Option<i32>,
    ) -> anyhow::Result<Vec<NotificationChannel>> {
        // ページネーションパラメータを共通Paginationサブメッセージで構成
        let request =
            tonic::Request::new(proto::k1s0::system::notification::v1::ListChannelsRequest {
                channel_type: channel_type.map(std::borrow::ToOwned::to_owned),
                enabled_only,
                pagination: Some(proto::k1s0::system::common::v1::Pagination {
                    page: page.unwrap_or(1),
                    page_size: page_size.unwrap_or(20),
                }),
            });

        let resp = self
            .client
            .clone()
            .list_channels(request)
            .await
            .map_err(|e| anyhow::anyhow!("NotificationService.ListChannels failed: {e}"))?
            .into_inner();

        Ok(resp
            .channels
            .into_iter()
            .map(Self::channel_from_proto)
            .collect())
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn create_channel(
        &self,
        name: &str,
        channel_type: &str,
        config_json: &str,
        enabled: bool,
    ) -> anyhow::Result<NotificationChannel> {
        let request = tonic::Request::new(
            proto::k1s0::system::notification::v1::CreateChannelRequest {
                name: name.to_owned(),
                channel_type: channel_type.to_owned(),
                config_json: config_json.to_owned(),
                enabled,
            },
        );

        let c = self
            .client
            .clone()
            .create_channel(request)
            .await
            .map_err(|e| anyhow::anyhow!("NotificationService.CreateChannel failed: {e}"))?
            .into_inner()
            .channel
            .ok_or_else(|| anyhow::anyhow!("empty channel in create response"))?;

        Ok(Self::channel_from_proto(c))
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn update_channel(
        &self,
        id: &str,
        name: Option<&str>,
        enabled: Option<bool>,
        config_json: Option<&str>,
    ) -> anyhow::Result<NotificationChannel> {
        let request = tonic::Request::new(
            proto::k1s0::system::notification::v1::UpdateChannelRequest {
                id: id.to_owned(),
                name: name.map(std::borrow::ToOwned::to_owned),
                enabled,
                config_json: config_json.map(std::borrow::ToOwned::to_owned),
            },
        );

        let c = self
            .client
            .clone()
            .update_channel(request)
            .await
            .map_err(|e| anyhow::anyhow!("NotificationService.UpdateChannel failed: {e}"))?
            .into_inner()
            .channel
            .ok_or_else(|| anyhow::anyhow!("empty channel in update response"))?;

        Ok(Self::channel_from_proto(c))
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn delete_channel(&self, id: &str) -> anyhow::Result<bool> {
        let request = tonic::Request::new(
            proto::k1s0::system::notification::v1::DeleteChannelRequest { id: id.to_owned() },
        );

        let resp = self
            .client
            .clone()
            .delete_channel(request)
            .await
            .map_err(|e| anyhow::anyhow!("NotificationService.DeleteChannel failed: {e}"))?
            .into_inner();

        Ok(resp.success)
    }

    // ── Template ──

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn get_template(&self, id: &str) -> anyhow::Result<Option<NotificationTemplate>> {
        let request =
            tonic::Request::new(proto::k1s0::system::notification::v1::GetTemplateRequest {
                id: id.to_owned(),
            });

        match self.client.clone().get_template(request).await {
            Ok(resp) => {
                // テンプレートが存在しない場合は None を返す
                let Some(t) = resp.into_inner().template else {
                    return Ok(None);
                };
                Ok(Some(Self::template_from_proto(t)))
            }
            Err(status) if status.code() == tonic::Code::NotFound => Ok(None),
            Err(e) => Err(anyhow::anyhow!(
                "NotificationService.GetTemplate failed: {e}"
            )),
        }
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn list_templates(
        &self,
        channel_type: Option<&str>,
        page: Option<i32>,
        page_size: Option<i32>,
    ) -> anyhow::Result<Vec<NotificationTemplate>> {
        // ページネーションパラメータを共通Paginationサブメッセージで構成
        let request = tonic::Request::new(
            proto::k1s0::system::notification::v1::ListTemplatesRequest {
                channel_type: channel_type.map(std::borrow::ToOwned::to_owned),
                pagination: Some(proto::k1s0::system::common::v1::Pagination {
                    page: page.unwrap_or(1),
                    page_size: page_size.unwrap_or(20),
                }),
            },
        );

        let resp = self
            .client
            .clone()
            .list_templates(request)
            .await
            .map_err(|e| anyhow::anyhow!("NotificationService.ListTemplates failed: {e}"))?
            .into_inner();

        Ok(resp
            .templates
            .into_iter()
            .map(Self::template_from_proto)
            .collect())
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn create_template(
        &self,
        name: &str,
        channel_type: &str,
        subject_template: Option<&str>,
        body_template: &str,
    ) -> anyhow::Result<NotificationTemplate> {
        let request = tonic::Request::new(
            proto::k1s0::system::notification::v1::CreateTemplateRequest {
                name: name.to_owned(),
                channel_type: channel_type.to_owned(),
                subject_template: subject_template.map(std::borrow::ToOwned::to_owned),
                body_template: body_template.to_owned(),
            },
        );

        let t = self
            .client
            .clone()
            .create_template(request)
            .await
            .map_err(|e| anyhow::anyhow!("NotificationService.CreateTemplate failed: {e}"))?
            .into_inner()
            .template
            .ok_or_else(|| anyhow::anyhow!("empty template in create response"))?;

        Ok(Self::template_from_proto(t))
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn update_template(
        &self,
        id: &str,
        name: Option<&str>,
        subject_template: Option<&str>,
        body_template: Option<&str>,
    ) -> anyhow::Result<NotificationTemplate> {
        let request = tonic::Request::new(
            proto::k1s0::system::notification::v1::UpdateTemplateRequest {
                id: id.to_owned(),
                name: name.map(std::borrow::ToOwned::to_owned),
                subject_template: subject_template.map(std::borrow::ToOwned::to_owned),
                body_template: body_template.map(std::borrow::ToOwned::to_owned),
            },
        );

        let t = self
            .client
            .clone()
            .update_template(request)
            .await
            .map_err(|e| anyhow::anyhow!("NotificationService.UpdateTemplate failed: {e}"))?
            .into_inner()
            .template
            .ok_or_else(|| anyhow::anyhow!("empty template in update response"))?;

        Ok(Self::template_from_proto(t))
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn delete_template(&self, id: &str) -> anyhow::Result<bool> {
        let request = tonic::Request::new(
            proto::k1s0::system::notification::v1::DeleteTemplateRequest { id: id.to_owned() },
        );

        let resp = self
            .client
            .clone()
            .delete_template(request)
            .await
            .map_err(|e| anyhow::anyhow!("NotificationService.DeleteTemplate failed: {e}"))?
            .into_inner();

        Ok(resp.success)
    }

    // ── Health Check ──

    /// gRPC Health Check Protocol を使ってサービスの疎通確認を行う。
    /// Bearer token なしで接続できるため readyz ヘルスチェックに適している。
    /// tonic-health サービスが登録されているサーバーに対して Check RPC を送信する。
    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn health_check(&self) -> anyhow::Result<()> {
        let channel = Channel::from_shared(self.address.clone())?
            .timeout(Duration::from_millis(self.timeout_ms))
            .connect_lazy();
        let mut health_client = tonic_health::pb::health_client::HealthClient::new(channel);
        let request = tonic::Request::new(tonic_health::pb::HealthCheckRequest {
            service: "k1s0.system.notification.v1.NotificationService".to_string(),
        });
        health_client
            .check(request)
            .await
            .map_err(|e| anyhow::anyhow!("notification gRPC Health Check 失敗: {e}"))?;
        Ok(())
    }
}
