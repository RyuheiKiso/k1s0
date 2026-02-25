//! tonic gRPC サービス実装。
//!
//! proto 生成コード (`src/proto/`) の NotificationService トレイトを実装する。
//! 各メソッドで proto 型 <-> 手動型の変換を行い、既存の NotificationGrpcService に委譲する。

use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::proto::k1s0::system::notification::v1::{
    notification_service_server::NotificationService,
    GetNotificationRequest as ProtoGetNotificationRequest,
    GetNotificationResponse as ProtoGetNotificationResponse,
    NotificationLog as ProtoNotificationLog,
    SendNotificationRequest as ProtoSendNotificationRequest,
    SendNotificationResponse as ProtoSendNotificationResponse,
};

use super::notification_grpc::{
    GetNotificationRequest, GrpcError, NotificationGrpcService, SendNotificationRequest,
};

// --- GrpcError -> tonic::Status 変換 ---

impl From<GrpcError> for Status {
    fn from(e: GrpcError) -> Self {
        match e {
            GrpcError::NotFound(msg) => Status::not_found(msg),
            GrpcError::InvalidArgument(msg) => Status::invalid_argument(msg),
            GrpcError::ChannelDisabled(msg) => Status::failed_precondition(msg),
            GrpcError::Internal(msg) => Status::internal(msg),
        }
    }
}

// --- NotificationService tonic ラッパー ---

pub struct NotificationServiceTonic {
    inner: Arc<NotificationGrpcService>,
}

impl NotificationServiceTonic {
    pub fn new(inner: Arc<NotificationGrpcService>) -> Self {
        Self { inner }
    }
}

#[async_trait::async_trait]
impl NotificationService for NotificationServiceTonic {
    async fn send_notification(
        &self,
        request: Request<ProtoSendNotificationRequest>,
    ) -> Result<Response<ProtoSendNotificationResponse>, Status> {
        let inner = request.into_inner();
        let req = SendNotificationRequest {
            channel_id: inner.channel_id,
            template_id: inner.template_id,
            variables: inner.variables,
            recipient: inner.recipient,
            subject: inner.subject,
            body: inner.body,
        };
        let resp = self
            .inner
            .send_notification(req)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoSendNotificationResponse {
            notification_id: resp.notification_id,
            status: resp.status,
            created_at: resp.created_at,
        }))
    }

    async fn get_notification(
        &self,
        request: Request<ProtoGetNotificationRequest>,
    ) -> Result<Response<ProtoGetNotificationResponse>, Status> {
        let inner = request.into_inner();
        let req = GetNotificationRequest {
            notification_id: inner.notification_id,
        };
        let resp = self
            .inner
            .get_notification(req)
            .await
            .map_err(Into::<Status>::into)?;

        let n = resp.notification;
        Ok(Response::new(ProtoGetNotificationResponse {
            notification: Some(ProtoNotificationLog {
                id: n.id,
                channel_id: n.channel_id,
                channel_type: n.channel_type,
                template_id: n.template_id,
                recipient: n.recipient,
                subject: n.subject,
                body: n.body,
                status: n.status,
                retry_count: n.retry_count,
                error_message: n.error_message,
                sent_at: n.sent_at,
                created_at: n.created_at,
            }),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grpc_error_not_found_to_status() {
        let err = GrpcError::NotFound("notification not found".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::NotFound);
        assert!(status.message().contains("notification not found"));
    }

    #[test]
    fn test_grpc_error_invalid_argument_to_status() {
        let err = GrpcError::InvalidArgument("invalid channel_id".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::InvalidArgument);
        assert!(status.message().contains("invalid channel_id"));
    }

    #[test]
    fn test_grpc_error_channel_disabled_to_status() {
        let err = GrpcError::ChannelDisabled("channel disabled".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::FailedPrecondition);
        assert!(status.message().contains("channel disabled"));
    }

    #[test]
    fn test_grpc_error_internal_to_status() {
        let err = GrpcError::Internal("database error".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::Internal);
        assert!(status.message().contains("database error"));
    }
}
