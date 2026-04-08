//! DLQ gRPC サービス実装（ドメイン層ラッパー）。

use std::sync::Arc;
use uuid::Uuid;

use crate::domain::entity::DlqMessage;
use crate::usecase::{
    DeleteMessageUseCase, GetMessageUseCase, ListMessagesUseCase, RetryAllUseCase,
    RetryMessageUseCase,
};

/// `GrpcError` は gRPC レイヤーのエラー型。
#[derive(Debug)]
pub enum GrpcError {
    NotFound(String),
    InvalidArgument(String),
    FailedPrecondition(String),
    Internal(String),
}

/// `DlqGrpcService` は DLQ gRPC サービスのビジネスロジック層。
pub struct DlqGrpcService {
    pub list_messages_uc: Arc<ListMessagesUseCase>,
    pub get_message_uc: Arc<GetMessageUseCase>,
    pub retry_message_uc: Arc<RetryMessageUseCase>,
    pub delete_message_uc: Arc<DeleteMessageUseCase>,
    pub retry_all_uc: Arc<RetryAllUseCase>,
}

impl DlqGrpcService {
    #[must_use] 
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

    /// CRIT-005 対応: `tenant_id` を渡して RLS でテナント分離しながら DLQ メッセージ一覧を取得する。
    pub async fn list_messages(
        &self,
        topic: &str,
        page: i32,
        page_size: i32,
        tenant_id: &str,
    ) -> Result<(Vec<DlqMessage>, i64), GrpcError> {
        self.list_messages_uc
            .execute(topic, page, page_size, tenant_id)
            .await
            .map_err(map_anyhow_to_grpc_error)
    }

    /// CRIT-005 対応: `tenant_id` を渡して RLS でテナント分離しながら DLQ メッセージを取得する。
    pub async fn get_message(&self, id: &str, tenant_id: &str) -> Result<DlqMessage, GrpcError> {
        let uuid = Uuid::parse_str(id)
            .map_err(|_| GrpcError::InvalidArgument(format!("invalid UUID: {id}")))?;
        self.get_message_uc
            .execute(uuid, tenant_id)
            .await
            .map_err(map_anyhow_to_grpc_error)
    }

    /// CRIT-005 対応: `tenant_id` を渡して RLS でテナント分離しながら DLQ メッセージをリトライする。
    pub async fn retry_message(&self, id: &str, tenant_id: &str) -> Result<DlqMessage, GrpcError> {
        let uuid = Uuid::parse_str(id)
            .map_err(|_| GrpcError::InvalidArgument(format!("invalid UUID: {id}")))?;
        self.retry_message_uc
            .execute(uuid, tenant_id)
            .await
            .map_err(map_anyhow_to_grpc_error)
    }

    /// CRIT-005 対応: `tenant_id` を渡して RLS でテナント分離しながら DLQ メッセージを削除する。
    pub async fn delete_message(&self, id: &str, tenant_id: &str) -> Result<(), GrpcError> {
        let uuid = Uuid::parse_str(id)
            .map_err(|_| GrpcError::InvalidArgument(format!("invalid UUID: {id}")))?;
        self.delete_message_uc
            .execute(uuid, tenant_id)
            .await
            .map_err(map_anyhow_to_grpc_error)
    }

    /// CRIT-005 対応: `tenant_id` を渡して RLS でテナント分離しながらトピック内の全 DLQ メッセージをリトライする。
    pub async fn retry_all(&self, topic: &str, tenant_id: &str) -> Result<i64, GrpcError> {
        self.retry_all_uc
            .execute(topic, tenant_id)
            .await
            .map_err(map_anyhow_to_grpc_error)
    }
}

/// `anyhow::Error` をドメインエラー型で型ベースに `GrpcError` へ変換する。
/// ダウンキャストに失敗した場合は internal エラーとする。
fn map_anyhow_to_grpc_error(err: anyhow::Error) -> GrpcError {
    use crate::domain::error::DlqManagerError;

    match err.downcast::<DlqManagerError>() {
        Ok(domain_err) => {
            let msg = domain_err.to_string();
            match domain_err {
                DlqManagerError::NotFound(_) => GrpcError::NotFound(msg),
                // HIGH-001 監査対応: 同一ボディのmatchアームをORパターンで結合
                DlqManagerError::ProcessFailed(_) | DlqManagerError::Internal(_) => {
                    GrpcError::Internal(msg)
                }
                DlqManagerError::AlreadyProcessed(_) => GrpcError::FailedPrecondition(msg),
                DlqManagerError::ValidationFailed(_) => GrpcError::InvalidArgument(msg),
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
