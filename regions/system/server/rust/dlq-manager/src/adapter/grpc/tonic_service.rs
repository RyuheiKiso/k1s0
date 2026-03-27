//! tonic gRPC サービス実装。

// §2.2 監査対応: ADR-0034 dual-write パターンで deprecated な status 文字列フィールドと
// 新 status_enum フィールドを同時設定するため、このファイル全体で deprecated 警告を抑制する。
#![allow(deprecated)]

use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::proto::k1s0::system::common::v1::PaginationResult as ProtoPaginationResult;
use crate::proto::k1s0::system::dlq::v1::{
    dlq_service_server::DlqService, DeleteMessageRequest as ProtoDeleteMessageRequest,
    DeleteMessageResponse as ProtoDeleteMessageResponse, DlqMessage as ProtoDlqMessage,
    GetMessageRequest as ProtoGetMessageRequest, GetMessageResponse as ProtoGetMessageResponse,
    ListMessagesRequest as ProtoListMessagesRequest,
    ListMessagesResponse as ProtoListMessagesResponse, RetryAllRequest as ProtoRetryAllRequest,
    RetryAllResponse as ProtoRetryAllResponse, RetryMessageRequest as ProtoRetryMessageRequest,
    RetryMessageResponse as ProtoRetryMessageResponse,
};

use super::dlq_grpc::{DlqGrpcService, GrpcError};

// --- GrpcError -> tonic::Status 変換 ---

impl From<GrpcError> for Status {
    fn from(e: GrpcError) -> Self {
        match e {
            GrpcError::NotFound(msg) => Status::not_found(msg),
            GrpcError::InvalidArgument(msg) => Status::invalid_argument(msg),
            GrpcError::FailedPrecondition(msg) => Status::failed_precondition(msg),
            GrpcError::Internal(msg) => Status::internal(msg),
        }
    }
}

// --- DlqMessage 変換ヘルパー ---

fn domain_to_proto(msg: crate::domain::entity::DlqMessage) -> ProtoDlqMessage {
    use crate::proto::k1s0::system::common::v1::Timestamp;
    let payload = serde_json::to_vec(&msg.payload).unwrap_or_default();
    ProtoDlqMessage {
        id: msg.id.to_string(),
        original_topic: msg.original_topic,
        error_message: msg.error_message,
        retry_count: msg.retry_count,
        max_retries: msg.max_retries,
        payload,
        status: msg.status.to_string(),
        created_at: Some(Timestamp {
            seconds: msg.created_at.timestamp(),
            nanos: msg.created_at.timestamp_subsec_nanos() as i32,
        }),
        updated_at: Some(Timestamp {
            seconds: msg.updated_at.timestamp(),
            nanos: msg.updated_at.timestamp_subsec_nanos() as i32,
        }),
        last_retry_at: msg.last_retry_at.map(|t| Timestamp {
            seconds: t.timestamp(),
            nanos: t.timestamp_subsec_nanos() as i32,
        }),
        // 後方互換フィールド（0 = UNSPECIFIED）
        status_enum: 0,
    }
}

// --- DlqServiceTonic ラッパー ---

pub struct DlqServiceTonic {
    inner: Arc<DlqGrpcService>,
}

impl DlqServiceTonic {
    pub fn new(inner: Arc<DlqGrpcService>) -> Self {
        Self { inner }
    }
}

#[async_trait::async_trait]
impl DlqService for DlqServiceTonic {
    async fn list_messages(
        &self,
        request: Request<ProtoListMessagesRequest>,
    ) -> Result<Response<ProtoListMessagesResponse>, Status> {
        let inner = request.into_inner();
        // ページネーションパラメータを共通Paginationサブメッセージから取得
        let pagination = inner.pagination.unwrap_or_default();
        let page = if pagination.page < 1 {
            1
        } else {
            pagination.page
        };
        let page_size = if pagination.page_size < 1 {
            20
        } else {
            pagination.page_size
        };
        let (messages, total) = self
            .inner
            .list_messages(&inner.topic, page, page_size)
            .await
            .map_err(Into::<Status>::into)?;
        let has_next = (page as i64 * page_size as i64) < total;

        Ok(Response::new(ProtoListMessagesResponse {
            messages: messages.into_iter().map(domain_to_proto).collect(),
            pagination: Some(ProtoPaginationResult {
                total_count: total,
                page,
                page_size,
                has_next,
            }),
        }))
    }

    async fn get_message(
        &self,
        request: Request<ProtoGetMessageRequest>,
    ) -> Result<Response<ProtoGetMessageResponse>, Status> {
        let inner = request.into_inner();
        let msg = self
            .inner
            .get_message(&inner.id)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoGetMessageResponse {
            message: Some(domain_to_proto(msg)),
        }))
    }

    async fn retry_message(
        &self,
        request: Request<ProtoRetryMessageRequest>,
    ) -> Result<Response<ProtoRetryMessageResponse>, Status> {
        let inner = request.into_inner();
        let msg = self
            .inner
            .retry_message(&inner.id)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoRetryMessageResponse {
            message: Some(domain_to_proto(msg)),
        }))
    }

    async fn delete_message(
        &self,
        request: Request<ProtoDeleteMessageRequest>,
    ) -> Result<Response<ProtoDeleteMessageResponse>, Status> {
        let inner = request.into_inner();
        let id = inner.id.clone();
        self.inner
            .delete_message(&inner.id)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoDeleteMessageResponse { id }))
    }

    async fn retry_all(
        &self,
        request: Request<ProtoRetryAllRequest>,
    ) -> Result<Response<ProtoRetryAllResponse>, Status> {
        let inner = request.into_inner();
        let retried_count = self
            .inner
            .retry_all(&inner.topic)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoRetryAllResponse {
            retried_count: retried_count as i32,
            message: format!(
                "{} messages retried in topic {}",
                retried_count, inner.topic
            ),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grpc_error_not_found_to_status() {
        let err = GrpcError::NotFound("message not found".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::NotFound);
        assert!(status.message().contains("message not found"));
    }

    #[test]
    fn test_grpc_error_invalid_argument_to_status() {
        let err = GrpcError::InvalidArgument("invalid uuid".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::InvalidArgument);
        assert!(status.message().contains("invalid uuid"));
    }

    #[test]
    fn test_grpc_error_internal_to_status() {
        let err = GrpcError::Internal("database error".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::Internal);
        assert!(status.message().contains("database error"));
    }
}
