//! tonic gRPC サービス実装。

use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::proto::k1s0::system::dlq::v1::{
    dlq_service_server::DlqService,
    DlqMessage as ProtoDlqMessage,
    DeleteMessageRequest as ProtoDeleteMessageRequest,
    DeleteMessageResponse as ProtoDeleteMessageResponse,
    GetMessageRequest as ProtoGetMessageRequest,
    GetMessageResponse as ProtoGetMessageResponse,
    ListMessagesRequest as ProtoListMessagesRequest,
    ListMessagesResponse as ProtoListMessagesResponse,
    RetryAllRequest as ProtoRetryAllRequest,
    RetryAllResponse as ProtoRetryAllResponse,
    RetryMessageRequest as ProtoRetryMessageRequest,
    RetryMessageResponse as ProtoRetryMessageResponse,
};

use super::dlq_grpc::{DlqGrpcService, GrpcError};

// --- GrpcError -> tonic::Status 変換 ---

impl From<GrpcError> for Status {
    fn from(e: GrpcError) -> Self {
        match e {
            GrpcError::NotFound(msg) => Status::not_found(msg),
            GrpcError::InvalidArgument(msg) => Status::invalid_argument(msg),
            GrpcError::Internal(msg) => Status::internal(msg),
        }
    }
}

// --- DlqMessage 変換ヘルパー ---

fn domain_to_proto(msg: crate::domain::entity::DlqMessage) -> ProtoDlqMessage {
    use crate::proto::k1s0::system::common::v1::Timestamp;
    ProtoDlqMessage {
        id: msg.id.to_string(),
        original_topic: msg.original_topic,
        error_message: msg.error_message,
        retry_count: msg.retry_count,
        max_retries: msg.max_retries,
        payload_json: msg.payload.to_string(),
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
        let (messages, total) = self
            .inner
            .list_messages(&inner.topic, inner.page, inner.page_size)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoListMessagesResponse {
            messages: messages.into_iter().map(domain_to_proto).collect(),
            total,
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

        Ok(Response::new(ProtoDeleteMessageResponse {
            id,
        }))
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
