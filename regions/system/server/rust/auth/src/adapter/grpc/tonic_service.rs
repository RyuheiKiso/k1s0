//! tonic gRPC サービス実装。
//!
//! proto 生成コード (`src/proto/`) の AuthService / AuditService トレイトを実装する。
//! AuthGrpcService / AuditGrpcService が直接 proto 型を返すため、変換なしで委譲する。

use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::proto::k1s0::system::auth::v1::{
    audit_service_server::AuditService, auth_service_server::AuthService,
    CheckPermissionRequest, CheckPermissionResponse, GetUserRequest, GetUserResponse,
    GetUserRolesRequest, GetUserRolesResponse, ListUsersRequest, ListUsersResponse,
    RecordAuditLogRequest, RecordAuditLogResponse, SearchAuditLogsRequest,
    SearchAuditLogsResponse, ValidateTokenRequest, ValidateTokenResponse,
};

use super::audit_grpc::AuditGrpcService;
use super::auth_grpc::{AuthGrpcService, GrpcError};

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

/// AuthServiceTonic は tonic の AuthService として AuthGrpcService をラップする。
pub struct AuthServiceTonic {
    inner: Arc<AuthGrpcService>,
}

impl AuthServiceTonic {
    pub fn new(inner: Arc<AuthGrpcService>) -> Self {
        Self { inner }
    }
}

#[async_trait::async_trait]
impl AuthService for AuthServiceTonic {
    async fn validate_token(
        &self,
        request: Request<ValidateTokenRequest>,
    ) -> Result<Response<ValidateTokenResponse>, Status> {
        let resp = self
            .inner
            .validate_token(request.into_inner())
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(resp))
    }

    async fn get_user(
        &self,
        request: Request<GetUserRequest>,
    ) -> Result<Response<GetUserResponse>, Status> {
        let resp = self
            .inner
            .get_user(request.into_inner())
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(resp))
    }

    async fn list_users(
        &self,
        request: Request<ListUsersRequest>,
    ) -> Result<Response<ListUsersResponse>, Status> {
        let resp = self
            .inner
            .list_users(request.into_inner())
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(resp))
    }

    async fn get_user_roles(
        &self,
        request: Request<GetUserRolesRequest>,
    ) -> Result<Response<GetUserRolesResponse>, Status> {
        let resp = self
            .inner
            .get_user_roles(request.into_inner())
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(resp))
    }

    async fn check_permission(
        &self,
        request: Request<CheckPermissionRequest>,
    ) -> Result<Response<CheckPermissionResponse>, Status> {
        let resp = self
            .inner
            .check_permission(request.into_inner())
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(resp))
    }
}

// --- AuditService tonic ラッパー ---

/// AuditServiceTonic は tonic の AuditService として AuditGrpcService をラップする。
pub struct AuditServiceTonic {
    inner: Arc<AuditGrpcService>,
}

impl AuditServiceTonic {
    pub fn new(inner: Arc<AuditGrpcService>) -> Self {
        Self { inner }
    }
}

#[async_trait::async_trait]
impl AuditService for AuditServiceTonic {
    async fn record_audit_log(
        &self,
        request: Request<RecordAuditLogRequest>,
    ) -> Result<Response<RecordAuditLogResponse>, Status> {
        let resp = self
            .inner
            .record_audit_log(request.into_inner())
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(resp))
    }

    async fn search_audit_logs(
        &self,
        request: Request<SearchAuditLogsRequest>,
    ) -> Result<Response<SearchAuditLogsResponse>, Status> {
        let resp = self
            .inner
            .search_audit_logs(request.into_inner())
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(resp))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::claims::{Claims, RealmAccess};
    use crate::domain::repository::user_repository::MockUserRepository;
    use crate::infrastructure::MockTokenVerifier;
    use crate::usecase::get_user::GetUserUseCase;
    use crate::usecase::get_user_roles::GetUserRolesUseCase;
    use crate::usecase::list_users::ListUsersUseCase;
    use crate::usecase::validate_token::ValidateTokenUseCase;
    use std::collections::HashMap;

    #[test]
    fn test_grpc_error_not_found_to_status() {
        let err = GrpcError::NotFound("user not found: abc".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::NotFound);
        assert!(status.message().contains("user not found: abc"));
    }

    #[test]
    fn test_grpc_error_unauthenticated_to_status() {
        let err = GrpcError::Unauthenticated("invalid token".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::Unauthenticated);
        assert!(status.message().contains("invalid token"));
    }

    #[test]
    fn test_grpc_error_invalid_argument_to_status() {
        let err = GrpcError::InvalidArgument("namespace is required".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::InvalidArgument);
        assert!(status.message().contains("namespace is required"));
    }

    #[test]
    fn test_grpc_error_internal_to_status() {
        let err = GrpcError::Internal("database connection failed".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::Internal);
        assert!(status.message().contains("database connection failed"));
    }

    #[tokio::test]
    async fn test_auth_service_tonic_validate_token() {
        let mut mock_verifier = MockTokenVerifier::new();
        mock_verifier.expect_verify_token().returning(|_| {
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
        let get_user_roles_uc = Arc::new(GetUserRolesUseCase::new(user_repo.clone()));
        let list_users_uc = Arc::new(ListUsersUseCase::new(user_repo));

        let auth_svc = Arc::new(AuthGrpcService::new(
            validate_uc,
            get_user_uc,
            get_user_roles_uc,
            list_users_uc,
        ));
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
        let mock_verifier = MockTokenVerifier::new();
        let validate_uc = Arc::new(ValidateTokenUseCase::new(
            Arc::new(mock_verifier),
            "test-issuer".to_string(),
            "test-audience".to_string(),
        ));
        let user_repo = Arc::new(MockUserRepository::new());
        let get_user_uc = Arc::new(GetUserUseCase::new(user_repo.clone()));
        let get_user_roles_uc = Arc::new(GetUserRolesUseCase::new(user_repo.clone()));
        let list_users_uc = Arc::new(ListUsersUseCase::new(user_repo));

        let auth_svc = Arc::new(AuthGrpcService::new(
            validate_uc,
            get_user_uc,
            get_user_roles_uc,
            list_users_uc,
        ));
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
