//! DLQ gRPC サービス実装（ドメイン層ラッパー）。

use std::sync::Arc;
use uuid::Uuid;

use crate::domain::entity::DlqMessage;
use crate::usecase::{
    DeleteMessageUseCase, GetMessageUseCase, ListMessagesUseCase, RetryAllUseCase,
    RetryMessageUseCase,
};

/// GrpcError は gRPC レイヤーのエラー型。
#[derive(Debug)]
pub enum GrpcError {
    NotFound(String),
    InvalidArgument(String),
    FailedPrecondition(String),
    Internal(String),
}

/// DlqGrpcService は DLQ gRPC サービスのビジネスロジック層。
pub struct DlqGrpcService {
    pub list_messages_uc: Arc<ListMessagesUseCase>,
    pub get_message_uc: Arc<GetMessageUseCase>,
    pub retry_message_uc: Arc<RetryMessageUseCase>,
    pub delete_message_uc: Arc<DeleteMessageUseCase>,
    pub retry_all_uc: Arc<RetryAllUseCase>,
}

impl DlqGrpcService {
    pub fn new(
        list_messages_uc: Arc<ListMessagesUseCase>,
        get_message_uc: Arc<GetMessageUseCase>,
        retry_message_uc: Arc<RetryMessageUseCase>,
        delete_message_uc: Arc<DeleteMessageUseCase>,
        retry_all_uc: Arc<RetryAllUseCase>,
    ) -> Self {
        Self {
            list_messages_uc,
            get_message_uc,
            retry_message_uc,
            delete_message_uc,
            retry_all_uc,
        }
    }

    /// DLQメッセージ一覧を取得する。ドメインエラー型で型ベースにGrpcErrorへ変換する。
    pub async fn list_messages(
        &self,
        topic: &str,
        page: i32,
        page_size: i32,
    ) -> Result<(Vec<DlqMessage>, i64), GrpcError> {
        self.list_messages_uc
            .execute(topic, page, page_size)
            .await
            .map_err(map_anyhow_to_grpc_error)
    }

    /// DLQメッセージを取得する。ドメインエラー型で型ベースにGrpcErrorへ変換する。
    pub async fn get_message(&self, id: &str) -> Result<DlqMessage, GrpcError> {
        let uuid = Uuid::parse_str(id)
            .map_err(|_| GrpcError::InvalidArgument(format!("invalid UUID: {}", id)))?;
        self.get_message_uc
            .execute(uuid)
            .await
            .map_err(map_anyhow_to_grpc_error)
    }

    /// DLQメッセージをリトライする。ドメインエラー型で型ベースにGrpcErrorへ変換する。
    pub async fn retry_message(&self, id: &str) -> Result<DlqMessage, GrpcError> {
        let uuid = Uuid::parse_str(id)
            .map_err(|_| GrpcError::InvalidArgument(format!("invalid UUID: {}", id)))?;
        self.retry_message_uc
            .execute(uuid)
            .await
            .map_err(map_anyhow_to_grpc_error)
    }

    /// DLQメッセージを削除する。ドメインエラー型で型ベースにGrpcErrorへ変換する。
    pub async fn delete_message(&self, id: &str) -> Result<(), GrpcError> {
        let uuid = Uuid::parse_str(id)
            .map_err(|_| GrpcError::InvalidArgument(format!("invalid UUID: {}", id)))?;
        self.delete_message_uc
            .execute(uuid)
            .await
            .map_err(map_anyhow_to_grpc_error)
    }

    /// トピック内の全DLQメッセージをリトライする。
    pub async fn retry_all(&self, topic: &str) -> Result<i64, GrpcError> {
        self.retry_all_uc
            .execute(topic)
            .await
            .map_err(map_anyhow_to_grpc_error)
    }
}

/// anyhow::Error をドメインエラー型で型ベースに GrpcError へ変換する。
/// ダウンキャストに失敗した場合は internal エラーとする。
fn map_anyhow_to_grpc_error(err: anyhow::Error) -> GrpcError {
    use crate::domain::error::DlqManagerError;

    match err.downcast::<DlqManagerError>() {
        Ok(domain_err) => {
            let msg = domain_err.to_string();
            match domain_err {
                DlqManagerError::NotFound(_) => GrpcError::NotFound(msg),
                DlqManagerError::ProcessFailed(_) => GrpcError::Internal(msg),
                DlqManagerError::AlreadyProcessed(_) => GrpcError::FailedPrecondition(msg),
                DlqManagerError::ValidationFailed(_) => GrpcError::InvalidArgument(msg),
                DlqManagerError::Internal(_) => GrpcError::Internal(msg),
            }
        }
        Err(err) => GrpcError::Internal(err.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grpc_error_variants() {
        let e1 = GrpcError::NotFound("test".to_string());
        let e2 = GrpcError::InvalidArgument("test".to_string());
        let e3 = GrpcError::Internal("test".to_string());
        assert!(matches!(e1, GrpcError::NotFound(_)));
        assert!(matches!(e2, GrpcError::InvalidArgument(_)));
        assert!(matches!(e3, GrpcError::Internal(_)));
    }
}
