//! tonic gRPC サービス実装。
//!
//! proto 生成コード (`src/proto/`) の VaultService トレイトを実装する。
//! 各メソッドで proto 型 <-> 手動型の変換を行い、既存の VaultGrpcService に委譲する。

use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::proto::k1s0::system::vault::v1::{
    vault_service_server::VaultService,
    DeleteSecretRequest as ProtoDeleteSecretRequest,
    DeleteSecretResponse as ProtoDeleteSecretResponse,
    GetSecretRequest as ProtoGetSecretRequest, GetSecretResponse as ProtoGetSecretResponse,
    ListSecretsRequest as ProtoListSecretsRequest,
    ListSecretsResponse as ProtoListSecretsResponse, SetSecretRequest as ProtoSetSecretRequest,
    SetSecretResponse as ProtoSetSecretResponse,
};

use super::vault_grpc::{
    DeleteSecretRequest, GetSecretRequest, GrpcError, ListSecretsRequest, SetSecretRequest,
    VaultGrpcService,
};

// --- GrpcError -> tonic::Status 変換 ---

impl From<GrpcError> for Status {
    fn from(e: GrpcError) -> Self {
        match e {
            GrpcError::NotFound(msg) => Status::not_found(msg),
            GrpcError::PermissionDenied(msg) => Status::permission_denied(msg),
            GrpcError::Internal(msg) => Status::internal(msg),
        }
    }
}

// --- VaultService tonic ラッパー ---

pub struct VaultServiceTonic {
    inner: Arc<VaultGrpcService>,
}

impl VaultServiceTonic {
    pub fn new(inner: Arc<VaultGrpcService>) -> Self {
        Self { inner }
    }
}

#[async_trait::async_trait]
impl VaultService for VaultServiceTonic {
    async fn get_secret(
        &self,
        request: Request<ProtoGetSecretRequest>,
    ) -> Result<Response<ProtoGetSecretResponse>, Status> {
        let inner = request.into_inner();
        let version = if inner.version.is_empty() {
            None
        } else {
            Some(
                inner
                    .version
                    .parse::<i64>()
                    .map_err(|e| Status::invalid_argument(format!("invalid version: {}", e)))?,
            )
        };
        let req = GetSecretRequest {
            path: inner.path,
            version,
        };
        let resp = self
            .inner
            .get_secret(req)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoGetSecretResponse {
            data: resp.data,
            version: resp.version,
            created_at: None,
        }))
    }

    async fn set_secret(
        &self,
        request: Request<ProtoSetSecretRequest>,
    ) -> Result<Response<ProtoSetSecretResponse>, Status> {
        let inner = request.into_inner();
        let req = SetSecretRequest {
            path: inner.path,
            data: inner.data,
        };
        let resp = self
            .inner
            .set_secret(req)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoSetSecretResponse {
            version: resp.version,
            created_at: None,
        }))
    }

    async fn delete_secret(
        &self,
        request: Request<ProtoDeleteSecretRequest>,
    ) -> Result<Response<ProtoDeleteSecretResponse>, Status> {
        let inner = request.into_inner();
        let req = DeleteSecretRequest {
            path: inner.path,
            versions: inner.versions,
        };
        self.inner
            .delete_secret(req)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoDeleteSecretResponse { success: true }))
    }

    async fn list_secrets(
        &self,
        request: Request<ProtoListSecretsRequest>,
    ) -> Result<Response<ProtoListSecretsResponse>, Status> {
        let req = ListSecretsRequest {
            path_prefix: request.into_inner().path_prefix,
        };
        let resp = self
            .inner
            .list_secrets(req)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoListSecretsResponse { keys: resp.keys }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grpc_error_not_found_to_status() {
        let err = GrpcError::NotFound("secret not found".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::NotFound);
        assert!(status.message().contains("secret not found"));
    }

    #[test]
    fn test_grpc_error_permission_denied_to_status() {
        let err = GrpcError::PermissionDenied("access denied".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::PermissionDenied);
    }

    #[test]
    fn test_grpc_error_internal_to_status() {
        let err = GrpcError::Internal("internal error".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::Internal);
    }
}
