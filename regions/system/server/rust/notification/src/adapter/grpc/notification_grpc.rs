use std::sync::Arc;

use uuid::Uuid;

use crate::domain::repository::NotificationLogRepository;
use crate::usecase::send_notification::{
    SendNotificationError, SendNotificationInput, SendNotificationUseCase,
};

// --- gRPC Request/Response Types ---

#[derive(Debug, Clone)]
pub struct SendNotificationRequest {
    pub channel_id: String,
    pub template_id: Option<String>,
    pub variables: std::collections::HashMap<String, String>,
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

// --- gRPC Error ---

#[derive(Debug, thiserror::Error)]
pub enum GrpcError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    #[error("channel disabled: {0}")]
    ChannelDisabled(String),

    #[error("internal: {0}")]
    Internal(String),
}

// --- NotificationGrpcService ---

pub struct NotificationGrpcService {
    send_notification_uc: Arc<SendNotificationUseCase>,
    log_repo: Arc<dyn NotificationLogRepository>,
}

impl NotificationGrpcService {
    pub fn new(
        send_notification_uc: Arc<SendNotificationUseCase>,
        log_repo: Arc<dyn NotificationLogRepository>,
    ) -> Self {
        Self {
            send_notification_uc,
            log_repo,
        }
    }

    pub async fn send_notification(
        &self,
        req: SendNotificationRequest,
    ) -> Result<SendNotificationResponse, GrpcError> {
        let channel_id = Uuid::parse_str(&req.channel_id)
            .map_err(|_| GrpcError::InvalidArgument(format!("invalid channel_id: {}", req.channel_id)))?;

        let body = req.body.unwrap_or_default();

        let input = SendNotificationInput {
            channel_id,
            recipient: req.recipient,
            subject: req.subject,
            body,
        };

        match self.send_notification_uc.execute(&input).await {
            Ok(output) => Ok(SendNotificationResponse {
                notification_id: output.log_id.to_string(),
                status: output.status,
                created_at: chrono::Utc::now().to_rfc3339(),
            }),
            Err(SendNotificationError::ChannelNotFound(id)) => {
                Err(GrpcError::NotFound(format!("channel not found: {}", id)))
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

        Ok(GetNotificationResponse {
            notification: PbNotificationLog {
                id: log.id.to_string(),
                channel_id: log.channel_id.to_string(),
                channel_type: String::new(),
                template_id: log.template_id.map(|id| id.to_string()),
                recipient: log.recipient,
                subject: log.subject,
                body: log.body,
                status: log.status,
                retry_count: 0,
                error_message: log.error_message,
                sent_at: log.sent_at.map(|t| t.to_rfc3339()),
                created_at: log.created_at.to_rfc3339(),
            },
        })
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
        let svc = NotificationGrpcService::new(
            Arc::new(SendNotificationUseCase::new(
                Arc::new(channel_mock),
                Arc::new(log_mock),
            )),
            log_repo_for_svc,
        );

        let req = SendNotificationRequest {
            channel_id: channel_id.to_string(),
            template_id: None,
            variables: std::collections::HashMap::new(),
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

        let svc = NotificationGrpcService::new(
            Arc::new(SendNotificationUseCase::new(
                Arc::new(channel_mock),
                Arc::new(log_mock),
            )),
            log_repo,
        );

        let req = SendNotificationRequest {
            channel_id: "not-a-uuid".to_string(),
            template_id: None,
            variables: std::collections::HashMap::new(),
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

        channel_mock
            .expect_find_by_id()
            .returning(|_| Ok(None));

        let missing_id = Uuid::new_v4();
        let svc = NotificationGrpcService::new(
            Arc::new(SendNotificationUseCase::new(
                Arc::new(channel_mock),
                Arc::new(log_mock),
            )),
            log_repo,
        );

        let req = SendNotificationRequest {
            channel_id: missing_id.to_string(),
            template_id: None,
            variables: std::collections::HashMap::new(),
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
        let return_log = log.clone();

        log_mock_for_repo
            .expect_find_by_id()
            .withf(move |id| *id == log_id)
            .returning(move |_| Ok(Some(return_log.clone())));

        let svc = NotificationGrpcService::new(
            Arc::new(SendNotificationUseCase::new(
                Arc::new(channel_mock),
                Arc::new(log_mock_for_uc),
            )),
            Arc::new(log_mock_for_repo),
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

        log_mock_for_repo
            .expect_find_by_id()
            .returning(|_| Ok(None));

        let svc = NotificationGrpcService::new(
            Arc::new(SendNotificationUseCase::new(
                Arc::new(channel_mock),
                Arc::new(log_mock_for_uc),
            )),
            Arc::new(log_mock_for_repo),
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
