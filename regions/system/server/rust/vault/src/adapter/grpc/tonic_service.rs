//! tonic gRPC サービス実装。
//!
//! proto 生成コード (`src/proto/`) の VaultService トレイトを実装する。
//! 各メソッドで proto 型 <-> 手動型の変換を行い、既存の VaultGrpcService に委譲する。

use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::proto::k1s0::system::vault::v1::{
    AuditLogEntry as ProtoAuditLogEntry,
    GetSecretMetadataRequest as ProtoGetSecretMetadataRequest,
    GetSecretMetadataResponse as ProtoGetSecretMetadataResponse,
    ListAuditLogsRequest as ProtoListAuditLogsRequest,
    ListAuditLogsResponse as ProtoListAuditLogsResponse,
    RotateSecretRequest as ProtoRotateSecretRequest,
    RotateSecretResponse as ProtoRotateSecretResponse,
    vault_service_server::VaultService,
    DeleteSecretRequest as ProtoDeleteSecretRequest,
    DeleteSecretResponse as ProtoDeleteSecretResponse,
    GetSecretRequest as ProtoGetSecretRequest, GetSecretResponse as ProtoGetSecretResponse,
    ListSecretsRequest as ProtoListSecretsRequest,
    ListSecretsResponse as ProtoListSecretsResponse, SetSecretRequest as ProtoSetSecretRequest,
    SetSecretResponse as ProtoSetSecretResponse,
};

use super::vault_grpc::{
    DeleteSecretRequest, GetSecretMetadataRequest, GetSecretRequest, GrpcError,
    ListAuditLogsRequest, ListSecretsRequest, RotateSecretRequest, SetSecretRequest,
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

fn to_proto_timestamp(dt: chrono::DateTime<chrono::Utc>) -> crate::proto::k1s0::system::common::v1::Timestamp {
    crate::proto::k1s0::system::common::v1::Timestamp {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos() as i32,
    }
}

#[async_trait::async_trait]
impl VaultService for VaultServiceTonic {
    async fn get_secret(
        &self,
        request: Request<ProtoGetSecretRequest>,
    ) -> Result<Response<ProtoGetSecretResponse>, Status> {
        let inner = request.into_inner();
        let version = if inner.version > 0 {
            Some(inner.version)
        } else {
            None
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
            created_at: Some(to_proto_timestamp(resp.created_at)),
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
            created_at: Some(to_proto_timestamp(resp.created_at)),
        }))
    }

    async fn rotate_secret(
        &self,
        request: Request<ProtoRotateSecretRequest>,
    ) -> Result<Response<ProtoRotateSecretResponse>, Status> {
        let inner = request.into_inner();
        let req = RotateSecretRequest {
            path: inner.path,
            data: inner.data,
        };
        let resp = self
            .inner
            .rotate_secret(req)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoRotateSecretResponse {
            path: resp.path,
            new_version: resp.new_version,
            rotated: resp.rotated,
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

    async fn get_secret_metadata(
        &self,
        request: Request<ProtoGetSecretMetadataRequest>,
    ) -> Result<Response<ProtoGetSecretMetadataResponse>, Status> {
        let req = GetSecretMetadataRequest {
            path: request.into_inner().path,
        };
        let resp = self
            .inner
            .get_secret_metadata(req)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoGetSecretMetadataResponse {
            path: resp.path,
            current_version: resp.current_version,
            version_count: resp.version_count,
            created_at: Some(to_proto_timestamp(resp.created_at)),
            updated_at: Some(to_proto_timestamp(resp.updated_at)),
        }))
    }

    async fn list_audit_logs(
        &self,
        request: Request<ProtoListAuditLogsRequest>,
    ) -> Result<Response<ProtoListAuditLogsResponse>, Status> {
        let inner = request.into_inner();
        let req = ListAuditLogsRequest {
            offset: inner.offset,
            limit: inner.limit,
        };
        let resp = self
            .inner
            .list_audit_logs(req)
            .await
            .map_err(Into::<Status>::into)?;

        let logs = resp
            .logs
            .into_iter()
            .map(|log| ProtoAuditLogEntry {
                id: log.id,
                key_path: log.key_path,
                action: log.action,
                actor_id: log.actor_id,
                ip_address: log.ip_address,
                success: log.success,
                error_msg: log.error_msg,
                created_at: Some(to_proto_timestamp(log.created_at)),
            })
            .collect();

        Ok(Response::new(ProtoListAuditLogsResponse { logs }))
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
