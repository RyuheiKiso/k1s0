//! tonic gRPC サービス実装。
//!
//! proto 生成コード (`src/proto/`) の NotificationService トレイトを実装する。
//! 各メソッドで proto 型 <-> 手動型の変換を行い、既存の NotificationGrpcService に委譲する。

// §2.2 監査対応: ADR-0034 dual-write パターンで deprecated な status 文字列フィールドと
// 新 status_enum フィールドを同時設定するため、このファイル全体で deprecated 警告を抑制する。
#![allow(deprecated)]

use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::proto::k1s0::system::common::v1::{
    PaginationResult as ProtoPaginationResult, Timestamp as ProtoTimestamp,
};
use crate::proto::k1s0::system::notification::v1::{
    notification_service_server::NotificationService, Channel as ProtoChannel,
    CreateChannelRequest as ProtoCreateChannelRequest,
    CreateChannelResponse as ProtoCreateChannelResponse,
    CreateTemplateRequest as ProtoCreateTemplateRequest,
    CreateTemplateResponse as ProtoCreateTemplateResponse,
    DeleteChannelRequest as ProtoDeleteChannelRequest,
    DeleteChannelResponse as ProtoDeleteChannelResponse,
    DeleteTemplateRequest as ProtoDeleteTemplateRequest,
    DeleteTemplateResponse as ProtoDeleteTemplateResponse,
    GetChannelRequest as ProtoGetChannelRequest, GetChannelResponse as ProtoGetChannelResponse,
    GetNotificationRequest as ProtoGetNotificationRequest,
    GetNotificationResponse as ProtoGetNotificationResponse,
    GetTemplateRequest as ProtoGetTemplateRequest, GetTemplateResponse as ProtoGetTemplateResponse,
    ListChannelsRequest as ProtoListChannelsRequest,
    ListChannelsResponse as ProtoListChannelsResponse,
    ListNotificationsRequest as ProtoListNotificationsRequest,
    ListNotificationsResponse as ProtoListNotificationsResponse,
    ListTemplatesRequest as ProtoListTemplatesRequest,
    ListTemplatesResponse as ProtoListTemplatesResponse, NotificationLog as ProtoNotificationLog,
    NotificationStatus, RetryNotificationRequest as ProtoRetryNotificationRequest,
    RetryNotificationResponse as ProtoRetryNotificationResponse,
    SendNotificationRequest as ProtoSendNotificationRequest,
    SendNotificationResponse as ProtoSendNotificationResponse, Template as ProtoTemplate,
    UpdateChannelRequest as ProtoUpdateChannelRequest,
    UpdateChannelResponse as ProtoUpdateChannelResponse,
    UpdateTemplateRequest as ProtoUpdateTemplateRequest,
    UpdateTemplateResponse as ProtoUpdateTemplateResponse,
};

use super::notification_grpc::{
    tenant_id_from_metadata, CreateChannelRequest, CreateTemplateRequest, DeleteChannelRequest,
    DeleteTemplateRequest, GetChannelRequest, GetNotificationRequest, GetTemplateRequest,
    GrpcError, ListChannelsRequest, ListNotificationsRequest, ListTemplatesRequest,
    NotificationGrpcService, RetryNotificationRequest, SendNotificationRequest,
    UpdateChannelRequest, UpdateTemplateRequest,
};

// --- ヘルパー関数 ---

/// ISO 8601 文字列を ProtoTimestamp に変換する。
/// パース失敗時は None を返す（既存の deprecated 文字列フィールドでカバー）。
fn str_to_ts(s: &str) -> Option<ProtoTimestamp> {
    chrono::DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt| ProtoTimestamp {
            seconds: dt.timestamp(),
            nanos: dt.timestamp_subsec_nanos() as i32,
        })
}

/// ドメインのステータス文字列を NotificationStatus enum の i32 値に変換する。
/// 未知の文字列は Unspecified (0) にフォールバックする。
fn status_str_to_enum(s: &str) -> i32 {
    match s {
        "pending" => NotificationStatus::Pending as i32,
        "sent" => NotificationStatus::Sent as i32,
        "failed" => NotificationStatus::Failed as i32,
        "retrying" => NotificationStatus::Retrying as i32,
        _ => NotificationStatus::Unspecified as i32,
    }
}

// --- GrpcError -> tonic::Status 変換 ---

impl From<GrpcError> for Status {
    fn from(e: GrpcError) -> Self {
        match e {
            GrpcError::NotFound(msg) => Status::not_found(msg),
            GrpcError::InvalidArgument(msg) => Status::invalid_argument(msg),
            GrpcError::FailedPrecondition(msg) => Status::failed_precondition(msg),
            GrpcError::ChannelDisabled(msg) => Status::failed_precondition(msg),
            GrpcError::Internal(msg) => Status::internal(msg),
        }
    }
}

fn channel_to_proto(ch: &super::notification_grpc::PbChannel) -> ProtoChannel {
    ProtoChannel {
        id: ch.id.clone(),
        name: ch.name.clone(),
        channel_type: ch.channel_type.clone(),
        config_json: ch.config_json.clone(),
        enabled: ch.enabled,
        // dual-write: 旧文字列フィールドと新 Timestamp 型フィールドを同時設定
        created_at: ch.created_at.clone(),
        updated_at: ch.updated_at.clone(),
        created_at_ts: str_to_ts(&ch.created_at),
        updated_at_ts: str_to_ts(&ch.updated_at),
    }
}

fn template_to_proto(t: &super::notification_grpc::PbTemplate) -> ProtoTemplate {
    ProtoTemplate {
        id: t.id.clone(),
        name: t.name.clone(),
        channel_type: t.channel_type.clone(),
        subject_template: t.subject_template.clone(),
        body_template: t.body_template.clone(),
        // dual-write: 旧文字列フィールドと新 Timestamp 型フィールドを同時設定
        created_at: t.created_at.clone(),
        updated_at: t.updated_at.clone(),
        created_at_ts: str_to_ts(&t.created_at),
        updated_at_ts: str_to_ts(&t.updated_at),
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
        // MEDIUM-RUST-001 監査対応: x-tenant-id メタデータからテナント ID を取得する（ADR-0028）。
        // 未設定の場合は UNAUTHENTICATED エラーを返す（フェイルクローズ設計）。
        let tenant_id = tenant_id_from_metadata(request.metadata())?;
        let inner = request.into_inner();
        let req = SendNotificationRequest {
            channel_id: inner.channel_id,
            tenant_id,
            template_id: inner.template_id,
            template_variables: inner.template_variables,
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
            // dual-write: 旧文字列フィールドと新 Timestamp 型フィールドを同時設定
            status: resp.status,
            created_at: resp.created_at.clone(),
            created_at_ts: str_to_ts(&resp.created_at),
        }))
    }

    async fn get_notification(
        &self,
        request: Request<ProtoGetNotificationRequest>,
    ) -> Result<Response<ProtoGetNotificationResponse>, Status> {
        // MEDIUM-RUST-001 監査対応: x-tenant-id メタデータからテナント ID を取得する（ADR-0028）。
        let tenant_id = tenant_id_from_metadata(request.metadata())?;
        let inner = request.into_inner();
        let req = GetNotificationRequest {
            notification_id: inner.notification_id,
            tenant_id,
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
                // dual-write: 旧文字列フィールドと新 enum/Timestamp フィールドを同時設定
                status: n.status.clone(),
                status_enum: status_str_to_enum(&n.status),
                retry_count: n.retry_count,
                error_message: n.error_message,
                sent_at: n.sent_at.clone(),
                sent_at_ts: n.sent_at.as_deref().and_then(str_to_ts),
                created_at: n.created_at.clone(),
                created_at_ts: str_to_ts(&n.created_at),
            }),
        }))
    }

    async fn retry_notification(
        &self,
        request: Request<ProtoRetryNotificationRequest>,
    ) -> Result<Response<ProtoRetryNotificationResponse>, Status> {
        // MEDIUM-RUST-001 監査対応: x-tenant-id メタデータからテナント ID を取得する（ADR-0028）。
        let tenant_id = tenant_id_from_metadata(request.metadata())?;
        let inner = request.into_inner();
        let req = RetryNotificationRequest {
            notification_id: inner.notification_id,
            tenant_id,
        };
        let resp = self
            .inner
            .retry_notification(req)
            .await
            .map_err(Into::<Status>::into)?;
        let n = resp.notification;
        Ok(Response::new(ProtoRetryNotificationResponse {
            notification: Some(ProtoNotificationLog {
                id: n.id,
                channel_id: n.channel_id,
                channel_type: n.channel_type,
                template_id: n.template_id,
                recipient: n.recipient,
                subject: n.subject,
                body: n.body,
                // dual-write: 旧文字列フィールドと新 enum/Timestamp フィールドを同時設定
                status: n.status.clone(),
                status_enum: status_str_to_enum(&n.status),
                retry_count: n.retry_count,
                error_message: n.error_message,
                sent_at: n.sent_at.clone(),
                sent_at_ts: n.sent_at.as_deref().and_then(str_to_ts),
                created_at: n.created_at.clone(),
                created_at_ts: str_to_ts(&n.created_at),
            }),
        }))
    }

    async fn list_notifications(
        &self,
        request: Request<ProtoListNotificationsRequest>,
    ) -> Result<Response<ProtoListNotificationsResponse>, Status> {
        let inner = request.into_inner();
        // ページネーションパラメータを共通Paginationサブメッセージから取得
        let pagination = inner.pagination.unwrap_or_default();
        let req = ListNotificationsRequest {
            channel_id: inner.channel_id,
            status: inner.status,
            page: if pagination.page <= 0 {
                1
            } else {
                pagination.page as u32
            },
            page_size: if pagination.page_size <= 0 {
                20
            } else {
                pagination.page_size as u32
            },
        };
        let resp = self
            .inner
            .list_notifications(req)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoListNotificationsResponse {
            notifications: resp
                .notifications
                .into_iter()
                .map(|n| ProtoNotificationLog {
                    id: n.id,
                    channel_id: n.channel_id,
                    channel_type: n.channel_type,
                    template_id: n.template_id,
                    recipient: n.recipient,
                    subject: n.subject,
                    body: n.body,
                    // dual-write: 旧文字列フィールドと新 enum/Timestamp フィールドを同時設定
                    status: n.status.clone(),
                    status_enum: status_str_to_enum(&n.status),
                    retry_count: n.retry_count,
                    error_message: n.error_message,
                    sent_at: n.sent_at.clone(),
                    sent_at_ts: n.sent_at.as_deref().and_then(str_to_ts),
                    created_at: n.created_at.clone(),
                    created_at_ts: str_to_ts(&n.created_at),
                })
                .collect(),
            pagination: Some(ProtoPaginationResult {
                total_count: resp.total as i64,
                page: resp.page as i32,
                page_size: resp.page_size as i32,
                has_next: resp.has_next,
            }),
        }))
    }

    async fn list_channels(
        &self,
        request: Request<ProtoListChannelsRequest>,
    ) -> Result<Response<ProtoListChannelsResponse>, Status> {
        // MEDIUM-RUST-001 監査対応: x-tenant-id メタデータからテナント ID を取得する（ADR-0028）。
        let tenant_id = tenant_id_from_metadata(request.metadata())?;
        let inner = request.into_inner();
        // ページネーションパラメータを共通Paginationサブメッセージから取得
        let pagination = inner.pagination.unwrap_or_default();
        let req = ListChannelsRequest {
            tenant_id,
            channel_type: inner.channel_type,
            enabled_only: inner.enabled_only,
            page: if pagination.page <= 0 {
                1
            } else {
                pagination.page as u32
            },
            page_size: if pagination.page_size <= 0 {
                20
            } else {
                pagination.page_size as u32
            },
        };
        let resp = self
            .inner
            .list_channels(req)
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(ProtoListChannelsResponse {
            channels: resp.channels.iter().map(channel_to_proto).collect(),
            pagination: Some(ProtoPaginationResult {
                total_count: resp.total as i64,
                page: resp.page as i32,
                page_size: resp.page_size as i32,
                has_next: resp.has_next,
            }),
        }))
    }

    async fn create_channel(
        &self,
        request: Request<ProtoCreateChannelRequest>,
    ) -> Result<Response<ProtoCreateChannelResponse>, Status> {
        // ADR-0028 Phase 1: gRPC メタデータ x-tenant-id からテナント ID を取得する。
        // CRIT-006 対応: フェイルクローズ設計。x-tenant-id が未設定の場合は UNAUTHENTICATED を返す。
        let tenant_id = tenant_id_from_metadata(request.metadata())?;
        let inner = request.into_inner();
        let req = CreateChannelRequest {
            name: inner.name,
            channel_type: inner.channel_type,
            config_json: if inner.config_json.is_empty() {
                None
            } else {
                Some(inner.config_json)
            },
            enabled: inner.enabled,
            tenant_id,
        };
        let resp = self
            .inner
            .create_channel(req)
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(ProtoCreateChannelResponse {
            channel: Some(channel_to_proto(&resp.channel)),
        }))
    }

    async fn get_channel(
        &self,
        request: Request<ProtoGetChannelRequest>,
    ) -> Result<Response<ProtoGetChannelResponse>, Status> {
        // MEDIUM-RUST-001 監査対応: x-tenant-id メタデータからテナント ID を取得する（ADR-0028）。
        let tenant_id = tenant_id_from_metadata(request.metadata())?;
        let inner = request.into_inner();
        let req = GetChannelRequest {
            id: inner.id,
            tenant_id,
        };
        let resp = self
            .inner
            .get_channel(req)
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(ProtoGetChannelResponse {
            channel: Some(channel_to_proto(&resp.channel)),
        }))
    }

    async fn update_channel(
        &self,
        request: Request<ProtoUpdateChannelRequest>,
    ) -> Result<Response<ProtoUpdateChannelResponse>, Status> {
        // MEDIUM-RUST-001 監査対応: x-tenant-id メタデータからテナント ID を取得する（ADR-0028）。
        let tenant_id = tenant_id_from_metadata(request.metadata())?;
        let inner = request.into_inner();
        let req = UpdateChannelRequest {
            id: inner.id,
            tenant_id,
            name: inner.name,
            enabled: inner.enabled,
            config_json: inner.config_json,
        };
        let resp = self
            .inner
            .update_channel(req)
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(ProtoUpdateChannelResponse {
            channel: Some(channel_to_proto(&resp.channel)),
        }))
    }

    async fn delete_channel(
        &self,
        request: Request<ProtoDeleteChannelRequest>,
    ) -> Result<Response<ProtoDeleteChannelResponse>, Status> {
        // MEDIUM-RUST-001 監査対応: x-tenant-id メタデータからテナント ID を取得する（ADR-0028）。
        let tenant_id = tenant_id_from_metadata(request.metadata())?;
        let inner = request.into_inner();
        let req = DeleteChannelRequest {
            id: inner.id,
            tenant_id,
        };
        let resp = self
            .inner
            .delete_channel(req)
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(ProtoDeleteChannelResponse {
            success: resp.success,
            message: resp.message,
        }))
    }

    async fn list_templates(
        &self,
        request: Request<ProtoListTemplatesRequest>,
    ) -> Result<Response<ProtoListTemplatesResponse>, Status> {
        let inner = request.into_inner();
        // ページネーションパラメータを共通Paginationサブメッセージから取得
        let pagination = inner.pagination.unwrap_or_default();
        let req = ListTemplatesRequest {
            channel_type: inner.channel_type,
            page: if pagination.page <= 0 {
                1
            } else {
                pagination.page as u32
            },
            page_size: if pagination.page_size <= 0 {
                20
            } else {
                pagination.page_size as u32
            },
        };
        let resp = self
            .inner
            .list_templates(req)
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(ProtoListTemplatesResponse {
            templates: resp.templates.iter().map(template_to_proto).collect(),
            pagination: Some(ProtoPaginationResult {
                total_count: resp.total as i64,
                page: resp.page as i32,
                page_size: resp.page_size as i32,
                has_next: resp.has_next,
            }),
        }))
    }

    async fn create_template(
        &self,
        request: Request<ProtoCreateTemplateRequest>,
    ) -> Result<Response<ProtoCreateTemplateResponse>, Status> {
        let inner = request.into_inner();
        let req = CreateTemplateRequest {
            name: inner.name,
            channel_type: inner.channel_type,
            subject_template: inner.subject_template,
            body_template: inner.body_template,
        };
        let resp = self
            .inner
            .create_template(req)
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(ProtoCreateTemplateResponse {
            template: Some(template_to_proto(&resp.template)),
        }))
    }

    async fn get_template(
        &self,
        request: Request<ProtoGetTemplateRequest>,
    ) -> Result<Response<ProtoGetTemplateResponse>, Status> {
        let inner = request.into_inner();
        let req = GetTemplateRequest { id: inner.id };
        let resp = self
            .inner
            .get_template(req)
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(ProtoGetTemplateResponse {
            template: Some(template_to_proto(&resp.template)),
        }))
    }

    async fn update_template(
        &self,
        request: Request<ProtoUpdateTemplateRequest>,
    ) -> Result<Response<ProtoUpdateTemplateResponse>, Status> {
        let inner = request.into_inner();
        let req = UpdateTemplateRequest {
            id: inner.id,
            name: inner.name,
            subject_template: inner.subject_template,
            body_template: inner.body_template,
        };
        let resp = self
            .inner
            .update_template(req)
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(ProtoUpdateTemplateResponse {
            template: Some(template_to_proto(&resp.template)),
        }))
    }

    async fn delete_template(
        &self,
        request: Request<ProtoDeleteTemplateRequest>,
    ) -> Result<Response<ProtoDeleteTemplateResponse>, Status> {
        let inner = request.into_inner();
        let req = DeleteTemplateRequest { id: inner.id };
        let resp = self
            .inner
            .delete_template(req)
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(ProtoDeleteTemplateResponse {
            success: resp.success,
            message: resp.message,
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
    fn test_grpc_error_failed_precondition_to_status() {
        let err = GrpcError::FailedPrecondition("already sent".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::FailedPrecondition);
        assert!(status.message().contains("already sent"));
    }

    #[test]
    fn test_grpc_error_internal_to_status() {
        let err = GrpcError::Internal("database error".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::Internal);
        assert!(status.message().contains("database error"));
    }
}
