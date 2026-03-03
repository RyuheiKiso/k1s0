use std::sync::Arc;

use uuid::Uuid;

use crate::domain::repository::NotificationChannelRepository;
use crate::domain::repository::NotificationLogRepository;
use crate::usecase::create_channel::{CreateChannelInput, CreateChannelUseCase};
use crate::usecase::create_template::{CreateTemplateInput, CreateTemplateUseCase};
use crate::usecase::delete_channel::DeleteChannelUseCase;
use crate::usecase::delete_template::DeleteTemplateUseCase;
use crate::usecase::get_channel::GetChannelUseCase;
use crate::usecase::get_template::GetTemplateUseCase;
use crate::usecase::list_channels::ListChannelsUseCase;
use crate::usecase::list_templates::ListTemplatesUseCase;
use crate::usecase::retry_notification::{
    RetryNotificationError, RetryNotificationInput, RetryNotificationUseCase,
};
use crate::usecase::send_notification::{
    SendNotificationError, SendNotificationInput, SendNotificationUseCase,
};
use crate::usecase::update_channel::{UpdateChannelInput, UpdateChannelUseCase};
use crate::usecase::update_template::{UpdateTemplateInput, UpdateTemplateUseCase};

// --- gRPC Request/Response Types ---

#[derive(Debug, Clone)]
pub struct SendNotificationRequest {
    pub channel_id: String,
    pub template_id: Option<String>,
    pub template_variables: std::collections::HashMap<String, String>,
    pub recipient: String,
    pub subject: Option<String>,
    pub body: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SendNotificationResponse {
    pub notification_id: String,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub struct GetNotificationRequest {
    pub notification_id: String,
}

#[derive(Debug, Clone)]
pub struct PbNotificationLog {
    pub id: String,
    pub channel_id: String,
    pub channel_type: String,
    pub template_id: Option<String>,
    pub recipient: String,
    pub subject: Option<String>,
    pub body: String,
    pub status: String,
    pub retry_count: u32,
    pub error_message: Option<String>,
    pub sent_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub struct GetNotificationResponse {
    pub notification: PbNotificationLog,
}

#[derive(Debug, Clone)]
pub struct RetryNotificationRequest {
    pub notification_id: String,
}

#[derive(Debug, Clone)]
pub struct RetryNotificationResponse {
    pub notification: PbNotificationLog,
}

#[derive(Debug, Clone)]
pub struct ListNotificationsRequest {
    pub channel_id: Option<String>,
    pub status: Option<String>,
    pub page: u32,
    pub page_size: u32,
}

#[derive(Debug, Clone)]
pub struct ListNotificationsResponse {
    pub notifications: Vec<PbNotificationLog>,
    pub total: u64,
    pub page: u32,
    pub page_size: u32,
    pub has_next: bool,
}

#[derive(Debug, Clone)]
pub struct PbChannel {
    pub id: String,
    pub name: String,
    pub channel_type: String,
    pub config_json: String,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone)]
pub struct ListChannelsRequest {
    pub channel_type: Option<String>,
    pub enabled_only: bool,
    pub page: u32,
    pub page_size: u32,
}

#[derive(Debug, Clone)]
pub struct ListChannelsResponse {
    pub channels: Vec<PbChannel>,
    pub total: u64,
    pub page: u32,
    pub page_size: u32,
    pub has_next: bool,
}

#[derive(Debug, Clone)]
pub struct CreateChannelRequest {
    pub name: String,
    pub channel_type: String,
    pub config_json: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub struct CreateChannelResponse {
    pub channel: PbChannel,
}

#[derive(Debug, Clone)]
pub struct GetChannelRequest {
    pub id: String,
}

#[derive(Debug, Clone)]
pub struct GetChannelResponse {
    pub channel: PbChannel,
}

#[derive(Debug, Clone)]
pub struct UpdateChannelRequest {
    pub id: String,
    pub name: Option<String>,
    pub enabled: Option<bool>,
    pub config_json: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UpdateChannelResponse {
    pub channel: PbChannel,
}

#[derive(Debug, Clone)]
pub struct DeleteChannelRequest {
    pub id: String,
}

#[derive(Debug, Clone)]
pub struct DeleteChannelResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct PbTemplate {
    pub id: String,
    pub name: String,
    pub channel_type: String,
    pub subject_template: Option<String>,
    pub body_template: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone)]
pub struct ListTemplatesRequest {
    pub channel_type: Option<String>,
    pub page: u32,
    pub page_size: u32,
}

#[derive(Debug, Clone)]
pub struct ListTemplatesResponse {
    pub templates: Vec<PbTemplate>,
    pub total: u64,
    pub page: u32,
    pub page_size: u32,
    pub has_next: bool,
}

#[derive(Debug, Clone)]
pub struct CreateTemplateRequest {
    pub name: String,
    pub channel_type: String,
    pub subject_template: Option<String>,
    pub body_template: String,
}

#[derive(Debug, Clone)]
pub struct CreateTemplateResponse {
    pub template: PbTemplate,
}

#[derive(Debug, Clone)]
pub struct GetTemplateRequest {
    pub id: String,
}

#[derive(Debug, Clone)]
pub struct GetTemplateResponse {
    pub template: PbTemplate,
}

#[derive(Debug, Clone)]
pub struct UpdateTemplateRequest {
    pub id: String,
    pub name: Option<String>,
    pub subject_template: Option<String>,
    pub body_template: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UpdateTemplateResponse {
    pub template: PbTemplate,
}

#[derive(Debug, Clone)]
pub struct DeleteTemplateRequest {
    pub id: String,
}

#[derive(Debug, Clone)]
pub struct DeleteTemplateResponse {
    pub success: bool,
    pub message: String,
}

// --- gRPC Error ---

#[derive(Debug, thiserror::Error)]
pub enum GrpcError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    #[error("failed precondition: {0}")]
    FailedPrecondition(String),

    #[error("channel disabled: {0}")]
    ChannelDisabled(String),

    #[error("internal: {0}")]
    Internal(String),
}

// --- NotificationGrpcService ---

pub struct NotificationGrpcService {
    send_notification_uc: Arc<SendNotificationUseCase>,
    retry_notification_uc: Option<Arc<RetryNotificationUseCase>>,
    log_repo: Arc<dyn NotificationLogRepository>,
    channel_repo: Arc<dyn NotificationChannelRepository>,
    create_channel_uc: Option<Arc<CreateChannelUseCase>>,
    list_channels_uc: Option<Arc<ListChannelsUseCase>>,
    get_channel_uc: Option<Arc<GetChannelUseCase>>,
    update_channel_uc: Option<Arc<UpdateChannelUseCase>>,
    delete_channel_uc: Option<Arc<DeleteChannelUseCase>>,
    create_template_uc: Option<Arc<CreateTemplateUseCase>>,
    list_templates_uc: Option<Arc<ListTemplatesUseCase>>,
    get_template_uc: Option<Arc<GetTemplateUseCase>>,
    update_template_uc: Option<Arc<UpdateTemplateUseCase>>,
    delete_template_uc: Option<Arc<DeleteTemplateUseCase>>,
}

impl NotificationGrpcService {
    pub fn new(
        send_notification_uc: Arc<SendNotificationUseCase>,
        log_repo: Arc<dyn NotificationLogRepository>,
        channel_repo: Arc<dyn NotificationChannelRepository>,
    ) -> Self {
        Self {
            send_notification_uc,
            retry_notification_uc: None,
            log_repo,
            channel_repo,
            create_channel_uc: None,
            list_channels_uc: None,
            get_channel_uc: None,
            update_channel_uc: None,
            delete_channel_uc: None,
            create_template_uc: None,
            list_templates_uc: None,
            get_template_uc: None,
            update_template_uc: None,
            delete_template_uc: None,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn with_management(
        send_notification_uc: Arc<SendNotificationUseCase>,
        retry_notification_uc: Arc<RetryNotificationUseCase>,
        log_repo: Arc<dyn NotificationLogRepository>,
        channel_repo: Arc<dyn NotificationChannelRepository>,
        create_channel_uc: Arc<CreateChannelUseCase>,
        list_channels_uc: Arc<ListChannelsUseCase>,
        get_channel_uc: Arc<GetChannelUseCase>,
        update_channel_uc: Arc<UpdateChannelUseCase>,
        delete_channel_uc: Arc<DeleteChannelUseCase>,
        create_template_uc: Arc<CreateTemplateUseCase>,
        list_templates_uc: Arc<ListTemplatesUseCase>,
        get_template_uc: Arc<GetTemplateUseCase>,
        update_template_uc: Arc<UpdateTemplateUseCase>,
        delete_template_uc: Arc<DeleteTemplateUseCase>,
    ) -> Self {
        Self {
            send_notification_uc,
            retry_notification_uc: Some(retry_notification_uc),
            log_repo,
            channel_repo,
            create_channel_uc: Some(create_channel_uc),
            list_channels_uc: Some(list_channels_uc),
            get_channel_uc: Some(get_channel_uc),
            update_channel_uc: Some(update_channel_uc),
            delete_channel_uc: Some(delete_channel_uc),
            create_template_uc: Some(create_template_uc),
            list_templates_uc: Some(list_templates_uc),
            get_template_uc: Some(get_template_uc),
            update_template_uc: Some(update_template_uc),
            delete_template_uc: Some(delete_template_uc),
        }
    }

    fn require<T>(opt: &Option<Arc<T>>, name: &str) -> Result<Arc<T>, GrpcError> {
        opt.clone().ok_or_else(|| {
            GrpcError::Internal(format!("{} usecase is not configured", name))
        })
    }

    pub async fn send_notification(
        &self,
        req: SendNotificationRequest,
    ) -> Result<SendNotificationResponse, GrpcError> {
        let channel_id = Uuid::parse_str(&req.channel_id)
            .map_err(|_| GrpcError::InvalidArgument(format!("invalid channel_id: {}", req.channel_id)))?;

        let template_id = req
            .template_id
            .as_deref()
            .map(Uuid::parse_str)
            .transpose()
            .map_err(|_| GrpcError::InvalidArgument("invalid template_id".to_string()))?;

        let body = req.body.unwrap_or_default();

        let template_variables = if req.template_variables.is_empty() {
            None
        } else {
            Some(req.template_variables)
        };

        let input = SendNotificationInput {
            channel_id,
            template_id,
            recipient: req.recipient,
            subject: req.subject,
            body,
            template_variables,
        };

        match self.send_notification_uc.execute(&input).await {
            Ok(output) => Ok(SendNotificationResponse {
                notification_id: output.log_id.to_string(),
                status: output.status,
                created_at: output.created_at.to_rfc3339(),
            }),
            Err(SendNotificationError::ChannelNotFound(id)) => {
                Err(GrpcError::NotFound(format!("channel not found: {}", id)))
            }
            Err(SendNotificationError::TemplateNotFound(id)) => {
                Err(GrpcError::NotFound(format!("template not found: {}", id)))
            }
            Err(SendNotificationError::ChannelDisabled(id)) => {
                Err(GrpcError::ChannelDisabled(format!("channel disabled: {}", id)))
            }
            Err(e) => Err(GrpcError::Internal(e.to_string())),
        }
    }

    pub async fn get_notification(
        &self,
        req: GetNotificationRequest,
    ) -> Result<GetNotificationResponse, GrpcError> {
        let id = Uuid::parse_str(&req.notification_id).map_err(|_| {
            GrpcError::InvalidArgument(format!("invalid notification_id: {}", req.notification_id))
        })?;

        let log = self
            .log_repo
            .find_by_id(&id)
            .await
            .map_err(|e| GrpcError::Internal(e.to_string()))?
            .ok_or_else(|| GrpcError::NotFound(format!("notification not found: {}", id)))?;

        let channel_type = match self
            .channel_repo
            .find_by_id(&log.channel_id)
            .await
            .map_err(|e| GrpcError::Internal(e.to_string()))?
        {
            Some(ch) => ch.channel_type,
            None => String::new(),
        };

        Ok(GetNotificationResponse {
            notification: log_to_pb(log, channel_type),
        })
    }

    pub async fn retry_notification(
        &self,
        req: RetryNotificationRequest,
    ) -> Result<RetryNotificationResponse, GrpcError> {
        let uc = Self::require(&self.retry_notification_uc, "retry_notification")?;
        let id = Uuid::parse_str(&req.notification_id).map_err(|_| {
            GrpcError::InvalidArgument(format!("invalid notification_id: {}", req.notification_id))
        })?;

        let log = uc
            .execute(&RetryNotificationInput {
                notification_id: id,
            })
            .await
            .map_err(|e| match e {
                RetryNotificationError::NotFound(id) => {
                    GrpcError::NotFound(format!("notification not found: {}", id))
                }
                RetryNotificationError::AlreadySent(id) => {
                    GrpcError::FailedPrecondition(format!("notification already sent: {}", id))
                }
                RetryNotificationError::ChannelNotFound(id) => {
                    GrpcError::NotFound(format!("channel not found: {}", id))
                }
                RetryNotificationError::Internal(msg) => GrpcError::Internal(msg),
            })?;

        let channel_type = self
            .channel_repo
            .find_by_id(&log.channel_id)
            .await
            .map_err(|e| GrpcError::Internal(e.to_string()))?
            .map(|ch| ch.channel_type)
            .unwrap_or_default();

        Ok(RetryNotificationResponse {
            notification: log_to_pb(log, channel_type),
        })
    }

    pub async fn list_notifications(
        &self,
        req: ListNotificationsRequest,
    ) -> Result<ListNotificationsResponse, GrpcError> {
        let page = if req.page == 0 { 1 } else { req.page };
        let page_size = if req.page_size == 0 { 20 } else { req.page_size };

        let channel_id = req
            .channel_id
            .as_deref()
            .map(Uuid::parse_str)
            .transpose()
            .map_err(|_| GrpcError::InvalidArgument("invalid channel_id".to_string()))?;

        let (logs, total) = self
            .log_repo
            .find_all_paginated(page, page_size, channel_id, req.status)
            .await
            .map_err(|e| GrpcError::Internal(e.to_string()))?;

        let mut notifications = Vec::with_capacity(logs.len());
        for log in logs {
            let channel_type = self
                .channel_repo
                .find_by_id(&log.channel_id)
                .await
                .map_err(|e| GrpcError::Internal(e.to_string()))?
                .map(|ch| ch.channel_type)
                .unwrap_or_default();
            notifications.push(log_to_pb(log, channel_type));
        }

        let has_next = (page as u64 * page_size as u64) < total;
        Ok(ListNotificationsResponse {
            notifications,
            total,
            page,
            page_size,
            has_next,
        })
    }

    pub async fn list_channels(
        &self,
        req: ListChannelsRequest,
    ) -> Result<ListChannelsResponse, GrpcError> {
        let uc = Self::require(&self.list_channels_uc, "list_channels")?;
        let page = if req.page == 0 { 1 } else { req.page };
        let page_size = if req.page_size == 0 { 20 } else { req.page_size };
        let (channels, total) = uc
            .execute_paginated(page, page_size, req.channel_type, req.enabled_only)
            .await
            .map_err(|e| GrpcError::Internal(e.to_string()))?;
        Ok(ListChannelsResponse {
            channels: channels.iter().map(channel_to_pb).collect(),
            total,
            page,
            page_size,
            has_next: (page as u64 * page_size as u64) < total,
        })
    }

    pub async fn create_channel(
        &self,
        req: CreateChannelRequest,
    ) -> Result<CreateChannelResponse, GrpcError> {
        let uc = Self::require(&self.create_channel_uc, "create_channel")?;
        let config = match req.config_json {
            Some(raw) if !raw.trim().is_empty() => serde_json::from_str::<serde_json::Value>(&raw)
                .map_err(|e| GrpcError::InvalidArgument(format!("invalid config_json: {}", e)))?,
            _ => serde_json::json!({}),
        };
        let input = CreateChannelInput {
            name: req.name,
            channel_type: req.channel_type,
            config,
            enabled: req.enabled,
        };
        let channel = uc
            .execute(&input)
            .await
            .map_err(|e| GrpcError::Internal(e.to_string()))?;
        Ok(CreateChannelResponse {
            channel: channel_to_pb(&channel),
        })
    }

    pub async fn get_channel(
        &self,
        req: GetChannelRequest,
    ) -> Result<GetChannelResponse, GrpcError> {
        let uc = Self::require(&self.get_channel_uc, "get_channel")?;
        let id = Uuid::parse_str(&req.id)
            .map_err(|_| GrpcError::InvalidArgument(format!("invalid channel id: {}", req.id)))?;
        let channel = uc.execute(&id).await.map_err(|e| {
            let msg = e.to_string();
            if msg.contains("not found") {
                GrpcError::NotFound(msg)
            } else {
                GrpcError::Internal(msg)
            }
        })?;
        Ok(GetChannelResponse {
            channel: channel_to_pb(&channel),
        })
    }

    pub async fn update_channel(
        &self,
        req: UpdateChannelRequest,
    ) -> Result<UpdateChannelResponse, GrpcError> {
        let uc = Self::require(&self.update_channel_uc, "update_channel")?;
        let id = Uuid::parse_str(&req.id)
            .map_err(|_| GrpcError::InvalidArgument(format!("invalid channel id: {}", req.id)))?;
        let config = match req.config_json {
            Some(raw) => Some(
                serde_json::from_str::<serde_json::Value>(&raw)
                    .map_err(|e| GrpcError::InvalidArgument(format!("invalid config_json: {}", e)))?,
            ),
            None => None,
        };
        let input = UpdateChannelInput {
            id,
            name: req.name,
            enabled: req.enabled,
            config,
        };
        let channel = uc.execute(&input).await.map_err(|e| {
            let msg = e.to_string();
            if msg.contains("not found") {
                GrpcError::NotFound(msg)
            } else {
                GrpcError::Internal(msg)
            }
        })?;
        Ok(UpdateChannelResponse {
            channel: channel_to_pb(&channel),
        })
    }

    pub async fn delete_channel(
        &self,
        req: DeleteChannelRequest,
    ) -> Result<DeleteChannelResponse, GrpcError> {
        let uc = Self::require(&self.delete_channel_uc, "delete_channel")?;
        let id = Uuid::parse_str(&req.id)
            .map_err(|_| GrpcError::InvalidArgument(format!("invalid channel id: {}", req.id)))?;
        uc.execute(&id).await.map_err(|e| {
            let msg = e.to_string();
            if msg.contains("not found") {
                GrpcError::NotFound(msg)
            } else {
                GrpcError::Internal(msg)
            }
        })?;
        Ok(DeleteChannelResponse {
            success: true,
            message: format!("channel {} deleted", id),
        })
    }

    pub async fn list_templates(
        &self,
        req: ListTemplatesRequest,
    ) -> Result<ListTemplatesResponse, GrpcError> {
        let uc = Self::require(&self.list_templates_uc, "list_templates")?;
        let page = if req.page == 0 { 1 } else { req.page };
        let page_size = if req.page_size == 0 { 20 } else { req.page_size };
        let (templates, total) = uc
            .execute_paginated(page, page_size, req.channel_type)
            .await
            .map_err(|e| GrpcError::Internal(e.to_string()))?;
        Ok(ListTemplatesResponse {
            templates: templates.iter().map(template_to_pb).collect(),
            total,
            page,
            page_size,
            has_next: (page as u64 * page_size as u64) < total,
        })
    }

    pub async fn create_template(
        &self,
        req: CreateTemplateRequest,
    ) -> Result<CreateTemplateResponse, GrpcError> {
        let uc = Self::require(&self.create_template_uc, "create_template")?;
        let input = CreateTemplateInput {
            name: req.name,
            channel_type: req.channel_type,
            subject_template: req.subject_template,
            body_template: req.body_template,
        };
        let template = uc
            .execute(&input)
            .await
            .map_err(|e| GrpcError::Internal(e.to_string()))?;
        Ok(CreateTemplateResponse {
            template: template_to_pb(&template),
        })
    }

    pub async fn get_template(
        &self,
        req: GetTemplateRequest,
    ) -> Result<GetTemplateResponse, GrpcError> {
        let uc = Self::require(&self.get_template_uc, "get_template")?;
        let id = Uuid::parse_str(&req.id)
            .map_err(|_| GrpcError::InvalidArgument(format!("invalid template id: {}", req.id)))?;
        let template = uc.execute(&id).await.map_err(|e| {
            let msg = e.to_string();
            if msg.contains("not found") {
                GrpcError::NotFound(msg)
            } else {
                GrpcError::Internal(msg)
            }
        })?;
        Ok(GetTemplateResponse {
            template: template_to_pb(&template),
        })
    }

    pub async fn update_template(
        &self,
        req: UpdateTemplateRequest,
    ) -> Result<UpdateTemplateResponse, GrpcError> {
        let uc = Self::require(&self.update_template_uc, "update_template")?;
        let id = Uuid::parse_str(&req.id)
            .map_err(|_| GrpcError::InvalidArgument(format!("invalid template id: {}", req.id)))?;
        let input = UpdateTemplateInput {
            id,
            name: req.name,
            subject_template: req.subject_template,
            body_template: req.body_template,
        };
        let template = uc.execute(&input).await.map_err(|e| {
            let msg = e.to_string();
            if msg.contains("not found") {
                GrpcError::NotFound(msg)
            } else {
                GrpcError::Internal(msg)
            }
        })?;
        Ok(UpdateTemplateResponse {
            template: template_to_pb(&template),
        })
    }

    pub async fn delete_template(
        &self,
        req: DeleteTemplateRequest,
    ) -> Result<DeleteTemplateResponse, GrpcError> {
        let uc = Self::require(&self.delete_template_uc, "delete_template")?;
        let id = Uuid::parse_str(&req.id)
            .map_err(|_| GrpcError::InvalidArgument(format!("invalid template id: {}", req.id)))?;
        uc.execute(&id).await.map_err(|e| {
            let msg = e.to_string();
            if msg.contains("not found") {
                GrpcError::NotFound(msg)
            } else {
                GrpcError::Internal(msg)
            }
        })?;
        Ok(DeleteTemplateResponse {
            success: true,
            message: format!("template {} deleted", id),
        })
    }
}

fn channel_to_pb(channel: &crate::domain::entity::notification_channel::NotificationChannel) -> PbChannel {
    PbChannel {
        id: channel.id.to_string(),
        name: channel.name.clone(),
        channel_type: channel.channel_type.clone(),
        config_json: channel.config.to_string(),
        enabled: channel.enabled,
        created_at: channel.created_at.to_rfc3339(),
        updated_at: channel.updated_at.to_rfc3339(),
    }
}

fn template_to_pb(template: &crate::domain::entity::notification_template::NotificationTemplate) -> PbTemplate {
    PbTemplate {
        id: template.id.to_string(),
        name: template.name.clone(),
        channel_type: template.channel_type.clone(),
        subject_template: template.subject_template.clone(),
        body_template: template.body_template.clone(),
        created_at: template.created_at.to_rfc3339(),
        updated_at: template.updated_at.to_rfc3339(),
    }
}

fn log_to_pb(log: crate::domain::entity::notification_log::NotificationLog, channel_type: String) -> PbNotificationLog {
    PbNotificationLog {
        id: log.id.to_string(),
        channel_id: log.channel_id.to_string(),
        channel_type,
        template_id: log.template_id.map(|id| id.to_string()),
        recipient: log.recipient,
        subject: log.subject,
        body: log.body,
        status: log.status,
        retry_count: log.retry_count,
        error_message: log.error_message,
        sent_at: log.sent_at.map(|t| t.to_rfc3339()),
        created_at: log.created_at.to_rfc3339(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::notification_channel::NotificationChannel;
    use crate::domain::entity::notification_log::NotificationLog;
    use crate::domain::repository::notification_channel_repository::MockNotificationChannelRepository;
    use crate::domain::repository::notification_log_repository::MockNotificationLogRepository;

    #[tokio::test]
    async fn test_send_notification_success() {
        let mut channel_mock = MockNotificationChannelRepository::new();
        let mut log_mock = MockNotificationLogRepository::new();

        let channel = NotificationChannel::new(
            "email".to_string(),
            "email".to_string(),
            serde_json::json!({}),
            true,
        );
        let channel_id = channel.id;
        let return_channel = channel.clone();

        channel_mock
            .expect_find_by_id()
            .withf(move |id| *id == channel_id)
            .returning(move |_| Ok(Some(return_channel.clone())));
        log_mock.expect_create().returning(|_| Ok(()));

        let log_repo_for_svc: Arc<dyn NotificationLogRepository> =
            Arc::new(MockNotificationLogRepository::new());
        let channel_repo_for_svc: Arc<dyn NotificationChannelRepository> =
            Arc::new(MockNotificationChannelRepository::new());
        let svc = NotificationGrpcService::new(
            Arc::new(SendNotificationUseCase::new(
                Arc::new(channel_mock),
                Arc::new(log_mock),
            )),
            log_repo_for_svc,
            channel_repo_for_svc,
        );

        let req = SendNotificationRequest {
            channel_id: channel_id.to_string(),
            template_id: None,
            template_variables: std::collections::HashMap::new(),
            recipient: "user@example.com".to_string(),
            subject: Some("Hello".to_string()),
            body: Some("Test message".to_string()),
        };
        let resp = svc.send_notification(req).await.unwrap();
        assert_eq!(resp.status, "sent");
        assert!(!resp.notification_id.is_empty());
    }

    #[tokio::test]
    async fn test_send_notification_invalid_channel_id() {
        let channel_mock = MockNotificationChannelRepository::new();
        let log_mock = MockNotificationLogRepository::new();
        let log_repo: Arc<dyn NotificationLogRepository> =
            Arc::new(MockNotificationLogRepository::new());
        let channel_repo: Arc<dyn NotificationChannelRepository> =
            Arc::new(MockNotificationChannelRepository::new());

        let svc = NotificationGrpcService::new(
            Arc::new(SendNotificationUseCase::new(
                Arc::new(channel_mock),
                Arc::new(log_mock),
            )),
            log_repo,
            channel_repo,
        );

        let req = SendNotificationRequest {
            channel_id: "not-a-uuid".to_string(),
            template_id: None,
            template_variables: std::collections::HashMap::new(),
            recipient: "user@example.com".to_string(),
            subject: None,
            body: Some("Test".to_string()),
        };
        let result = svc.send_notification(req).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            GrpcError::InvalidArgument(msg) => assert!(msg.contains("invalid channel_id")),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_send_notification_channel_not_found() {
        let mut channel_mock = MockNotificationChannelRepository::new();
        let log_mock = MockNotificationLogRepository::new();
        let log_repo: Arc<dyn NotificationLogRepository> =
            Arc::new(MockNotificationLogRepository::new());
        let channel_repo: Arc<dyn NotificationChannelRepository> =
            Arc::new(MockNotificationChannelRepository::new());

        channel_mock.expect_find_by_id().returning(|_| Ok(None));

        let missing_id = Uuid::new_v4();
        let svc = NotificationGrpcService::new(
            Arc::new(SendNotificationUseCase::new(
                Arc::new(channel_mock),
                Arc::new(log_mock),
            )),
            log_repo,
            channel_repo,
        );

        let req = SendNotificationRequest {
            channel_id: missing_id.to_string(),
            template_id: None,
            template_variables: std::collections::HashMap::new(),
            recipient: "user@example.com".to_string(),
            subject: None,
            body: Some("Test".to_string()),
        };
        let result = svc.send_notification(req).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            GrpcError::NotFound(msg) => assert!(msg.contains("channel not found")),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_get_notification_success() {
        let channel_mock = MockNotificationChannelRepository::new();
        let log_mock_for_uc = MockNotificationLogRepository::new();
        let mut log_mock_for_repo = MockNotificationLogRepository::new();

        let log = NotificationLog::new(
            Uuid::new_v4(),
            "user@example.com".to_string(),
            Some("Subject".to_string()),
            "Body".to_string(),
        );
        let log_id = log.id;
        let channel_id = log.channel_id;
        let return_log = log.clone();

        log_mock_for_repo
            .expect_find_by_id()
            .withf(move |id| *id == log_id)
            .returning(move |_| Ok(Some(return_log.clone())));

        let mut channel_mock_for_repo = MockNotificationChannelRepository::new();
        channel_mock_for_repo
            .expect_find_by_id()
            .withf(move |id| *id == channel_id)
            .returning(|_| {
                Ok(Some(NotificationChannel::new(
                    "test".to_string(),
                    "email".to_string(),
                    serde_json::json!({}),
                    true,
                )))
            });

        let svc = NotificationGrpcService::new(
            Arc::new(SendNotificationUseCase::new(
                Arc::new(channel_mock),
                Arc::new(log_mock_for_uc),
            )),
            Arc::new(log_mock_for_repo),
            Arc::new(channel_mock_for_repo),
        );

        let req = GetNotificationRequest {
            notification_id: log_id.to_string(),
        };
        let resp = svc.get_notification(req).await.unwrap();
        assert_eq!(resp.notification.id, log_id.to_string());
        assert_eq!(resp.notification.recipient, "user@example.com");
    }

    #[tokio::test]
    async fn test_get_notification_not_found() {
        let channel_mock = MockNotificationChannelRepository::new();
        let log_mock_for_uc = MockNotificationLogRepository::new();
        let mut log_mock_for_repo = MockNotificationLogRepository::new();

        log_mock_for_repo.expect_find_by_id().returning(|_| Ok(None));

        let channel_mock_for_repo = MockNotificationChannelRepository::new();

        let svc = NotificationGrpcService::new(
            Arc::new(SendNotificationUseCase::new(
                Arc::new(channel_mock),
                Arc::new(log_mock_for_uc),
            )),
            Arc::new(log_mock_for_repo),
            Arc::new(channel_mock_for_repo),
        );

        let req = GetNotificationRequest {
            notification_id: Uuid::new_v4().to_string(),
        };
        let result = svc.get_notification(req).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            GrpcError::NotFound(msg) => assert!(msg.contains("notification not found")),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
