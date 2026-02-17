//! tonic gRPC サービス登録ヘルパー。
//!
//! proto 生成コード (tonic-build) が未生成のため、
//! tonic::transport::Server に登録するためのサービスラッパーを手動で定義する。
//! tonic-build による生成後はこのファイルは不要となる。
//!
//! 現時点では、各 RPC メソッドは gRPC の標準的な protobuf シリアライゼーションではなく、
//! 手動定義の型を JSON でやり取りする。gRPC クライアントは content-type: application/grpc+json
//! を使用する必要がある。
//!
//! proto 生成後は、生成された XxxServer<T> に置き換えること。

use std::sync::Arc;

use tonic::{Request, Response, Status};

use super::auth_grpc::{
    AuthGrpcService, CheckPermissionRequest, CheckPermissionResponse, GetUserRequest,
    GetUserResponse, GetUserRolesRequest, GetUserRolesResponse, GrpcError, ListUsersRequest,
    ListUsersResponse, ValidateTokenRequest, ValidateTokenResponse,
};

use super::audit_grpc::{
    AuditGrpcService, RecordAuditLogGrpcRequest, RecordAuditLogGrpcResponse,
    SearchAuditLogsGrpcRequest, SearchAuditLogsGrpcResponse,
};

// --- GrpcError -> tonic::Status 変換 ---

impl From<GrpcError> for Status {
    fn from(e: GrpcError) -> Self {
        match e {
            GrpcError::NotFound(msg) => Status::not_found(msg),
            GrpcError::Unauthenticated(msg) => Status::unauthenticated(msg),
            GrpcError::InvalidArgument(msg) => Status::invalid_argument(msg),
            GrpcError::Internal(msg) => Status::internal(msg),
        }
    }
}

// --- AuthService tonic ラッパー ---

/// AuthServiceTonic は tonic の gRPC サービスとして AuthGrpcService をラップする。
pub struct AuthServiceTonic {
    inner: Arc<AuthGrpcService>,
}

impl AuthServiceTonic {
    pub fn new(inner: Arc<AuthGrpcService>) -> Self {
        Self { inner }
    }

    pub async fn validate_token(
        &self,
        request: Request<ValidateTokenRequest>,
    ) -> Result<Response<ValidateTokenResponse>, Status> {
        let resp = self.inner.validate_token(request.into_inner()).await?;
        Ok(Response::new(resp))
    }

    pub async fn get_user(
        &self,
        request: Request<GetUserRequest>,
    ) -> Result<Response<GetUserResponse>, Status> {
        let resp = self.inner.get_user(request.into_inner()).await?;
        Ok(Response::new(resp))
    }

    pub async fn list_users(
        &self,
        request: Request<ListUsersRequest>,
    ) -> Result<Response<ListUsersResponse>, Status> {
        let resp = self.inner.list_users(request.into_inner()).await?;
        Ok(Response::new(resp))
    }

    pub async fn get_user_roles(
        &self,
        request: Request<GetUserRolesRequest>,
    ) -> Result<Response<GetUserRolesResponse>, Status> {
        let resp = self.inner.get_user_roles(request.into_inner()).await?;
        Ok(Response::new(resp))
    }

    pub async fn check_permission(
        &self,
        request: Request<CheckPermissionRequest>,
    ) -> Result<Response<CheckPermissionResponse>, Status> {
        let resp = self.inner.check_permission(request.into_inner()).await?;
        Ok(Response::new(resp))
    }
}

// --- AuditService tonic ラッパー ---

/// AuditServiceTonic は tonic の gRPC サービスとして AuditGrpcService をラップする。
pub struct AuditServiceTonic {
    inner: Arc<AuditGrpcService>,
}

impl AuditServiceTonic {
    pub fn new(inner: Arc<AuditGrpcService>) -> Self {
        Self { inner }
    }

    pub async fn record_audit_log(
        &self,
        request: Request<RecordAuditLogGrpcRequest>,
    ) -> Result<Response<RecordAuditLogGrpcResponse>, Status> {
        let resp = self.inner.record_audit_log(request.into_inner()).await?;
        Ok(Response::new(resp))
    }

    pub async fn search_audit_logs(
        &self,
        request: Request<SearchAuditLogsGrpcRequest>,
    ) -> Result<Response<SearchAuditLogsGrpcResponse>, Status> {
        let resp = self
            .inner
            .search_audit_logs(request.into_inner())
            .await?;
        Ok(Response::new(resp))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tonic::Code;

    #[test]
    fn test_grpc_error_not_found_to_status() {
        let err = GrpcError::NotFound("user not found: abc".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), Code::NotFound);
        assert!(status.message().contains("user not found: abc"));
    }

    #[test]
    fn test_grpc_error_unauthenticated_to_status() {
        let err = GrpcError::Unauthenticated("invalid token".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), Code::Unauthenticated);
        assert!(status.message().contains("invalid token"));
    }

    #[test]
    fn test_grpc_error_invalid_argument_to_status() {
        let err = GrpcError::InvalidArgument("namespace is required".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), Code::InvalidArgument);
        assert!(status.message().contains("namespace is required"));
    }

    #[test]
    fn test_grpc_error_internal_to_status() {
        let err = GrpcError::Internal("database connection failed".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), Code::Internal);
        assert!(status.message().contains("database connection failed"));
    }

    #[tokio::test]
    async fn test_auth_service_tonic_validate_token() {
        use crate::domain::entity::claims::{Claims, RealmAccess};
        use crate::infrastructure::MockTokenVerifier;
        use crate::usecase::get_user::GetUserUseCase;
        use crate::usecase::list_users::ListUsersUseCase;
        use crate::usecase::validate_token::ValidateTokenUseCase;
        use crate::domain::repository::user_repository::MockUserRepository;
        use std::collections::HashMap;

        let mut mock_verifier = MockTokenVerifier::new();
        mock_verifier
            .expect_verify_token()
            .returning(|_| {
                Ok(Claims {
                    sub: "tonic-test-user".to_string(),
                    iss: "test-issuer".to_string(),
                    aud: "test-audience".to_string(),
                    exp: chrono::Utc::now().timestamp() + 3600,
                    iat: chrono::Utc::now().timestamp(),
                    jti: "test-jti".to_string(),
                    typ: "Bearer".to_string(),
                    azp: "test-client".to_string(),
                    scope: "openid".to_string(),
                    preferred_username: "tonic.user".to_string(),
                    email: "tonic@test.com".to_string(),
                    realm_access: RealmAccess {
                        roles: vec!["user".to_string()],
                    },
                    resource_access: HashMap::new(),
                    tier_access: vec!["system".to_string()],
                })
            });

        let validate_uc = Arc::new(ValidateTokenUseCase::new(
            Arc::new(mock_verifier),
            "test-issuer".to_string(),
            "test-audience".to_string(),
        ));
        let user_repo = Arc::new(MockUserRepository::new());
        let get_user_uc = Arc::new(GetUserUseCase::new(user_repo.clone()));
        let list_users_uc = Arc::new(ListUsersUseCase::new(user_repo));

        let auth_svc = Arc::new(AuthGrpcService::new(validate_uc, get_user_uc, list_users_uc));
        let tonic_svc = AuthServiceTonic::new(auth_svc);

        let req = Request::new(ValidateTokenRequest {
            token: "test-token".to_string(),
        });
        let resp = tonic_svc.validate_token(req).await.unwrap();
        let inner = resp.into_inner();

        assert!(inner.valid);
        let claims = inner.claims.unwrap();
        assert_eq!(claims.sub, "tonic-test-user");
        assert_eq!(claims.preferred_username, "tonic.user");
    }

    #[tokio::test]
    async fn test_auth_service_tonic_check_permission() {
        use crate::infrastructure::MockTokenVerifier;
        use crate::usecase::get_user::GetUserUseCase;
        use crate::usecase::list_users::ListUsersUseCase;
        use crate::usecase::validate_token::ValidateTokenUseCase;
        use crate::domain::repository::user_repository::MockUserRepository;

        let mock_verifier = MockTokenVerifier::new();
        let validate_uc = Arc::new(ValidateTokenUseCase::new(
            Arc::new(mock_verifier),
            "test-issuer".to_string(),
            "test-audience".to_string(),
        ));
        let user_repo = Arc::new(MockUserRepository::new());
        let get_user_uc = Arc::new(GetUserUseCase::new(user_repo.clone()));
        let list_users_uc = Arc::new(ListUsersUseCase::new(user_repo));

        let auth_svc = Arc::new(AuthGrpcService::new(validate_uc, get_user_uc, list_users_uc));
        let tonic_svc = AuthServiceTonic::new(auth_svc);

        // sys_admin should be allowed
        let req = Request::new(CheckPermissionRequest {
            user_id: "user-1".to_string(),
            permission: "admin".to_string(),
            resource: "configs".to_string(),
            roles: vec!["sys_admin".to_string()],
        });
        let resp = tonic_svc.check_permission(req).await.unwrap();
        assert!(resp.into_inner().allowed);

        // regular user should be denied
        let req = Request::new(CheckPermissionRequest {
            user_id: "user-1".to_string(),
            permission: "admin".to_string(),
            resource: "configs".to_string(),
            roles: vec!["user".to_string()],
        });
        let resp = tonic_svc.check_permission(req).await.unwrap();
        let inner = resp.into_inner();
        assert!(!inner.allowed);
        assert!(inner.reason.contains("insufficient permissions"));
    }
}
