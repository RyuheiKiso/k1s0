//! gRPCサービス実装

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use tonic::{Request, Response, Status};

use crate::application::AuthService;
use crate::domain::{PermissionRepository, RoleRepository, TokenRepository, UserRepository};

// Generated code from proto
pub mod auth_v1 {
    tonic::include_proto!("k1s0.auth.v1");
}

use auth_v1::auth_service_server::AuthService as AuthServiceTrait;
use auth_v1::{
    AuthenticateRequest, AuthenticateResponse, CheckPermissionRequest, CheckPermissionResponse,
    GetUserRequest, GetUserResponse, ListUserRolesRequest, ListUserRolesResponse,
    RefreshTokenRequest, RefreshTokenResponse, Role as ProtoRole, User as ProtoUser,
};

/// gRPCサービス実装
pub struct GrpcAuthService<U, R, P, T>
where
    U: UserRepository,
    R: RoleRepository,
    P: PermissionRepository,
    T: TokenRepository,
{
    service: Arc<AuthService<U, R, P, T>>,
}

impl<U, R, P, T> GrpcAuthService<U, R, P, T>
where
    U: UserRepository,
    R: RoleRepository,
    P: PermissionRepository,
    T: TokenRepository,
{
    /// 新しいgRPCサービスを作成
    pub fn new(service: Arc<AuthService<U, R, P, T>>) -> Self {
        Self { service }
    }
}

fn system_time_to_rfc3339(time: SystemTime) -> String {
    let duration = time.duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = duration.as_secs();
    let datetime = chrono::DateTime::from_timestamp(secs as i64, 0)
        .unwrap_or_else(|| chrono::DateTime::from_timestamp(0, 0).unwrap());
    datetime.format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

fn option_system_time_to_rfc3339(time: Option<SystemTime>) -> String {
    time.map(system_time_to_rfc3339).unwrap_or_default()
}

fn user_to_proto(user: crate::domain::User) -> ProtoUser {
    ProtoUser {
        user_id: user.user_id,
        login_id: user.login_id,
        email: user.email,
        display_name: user.display_name,
        status: user.status.to_i32(),
        last_login_at: option_system_time_to_rfc3339(user.last_login_at),
        created_at: system_time_to_rfc3339(user.created_at),
        updated_at: system_time_to_rfc3339(user.updated_at),
    }
}

fn role_to_proto(role: crate::domain::Role) -> ProtoRole {
    ProtoRole {
        role_id: role.role_id,
        role_name: role.role_name,
        description: role.description,
    }
}

#[tonic::async_trait]
impl<U, R, P, T> AuthServiceTrait for GrpcAuthService<U, R, P, T>
where
    U: UserRepository,
    R: RoleRepository,
    P: PermissionRepository,
    T: TokenRepository,
{
    async fn authenticate(
        &self,
        request: Request<AuthenticateRequest>,
    ) -> Result<Response<AuthenticateResponse>, Status> {
        let req = request.into_inner();

        let token = self
            .service
            .authenticate(&req.login_id, &req.password)
            .await
            .map_err(|e| Status::new(e.to_grpc_code().into(), e.to_string()))?;

        Ok(Response::new(AuthenticateResponse {
            access_token: token.access_token,
            refresh_token: token.refresh_token,
            expires_in: token.expires_in,
            token_type: token.token_type,
        }))
    }

    async fn refresh_token(
        &self,
        request: Request<RefreshTokenRequest>,
    ) -> Result<Response<RefreshTokenResponse>, Status> {
        let req = request.into_inner();

        let token = self
            .service
            .refresh_token(&req.refresh_token)
            .await
            .map_err(|e| Status::new(e.to_grpc_code().into(), e.to_string()))?;

        Ok(Response::new(RefreshTokenResponse {
            access_token: token.access_token,
            refresh_token: token.refresh_token,
            expires_in: token.expires_in,
        }))
    }

    async fn check_permission(
        &self,
        request: Request<CheckPermissionRequest>,
    ) -> Result<Response<CheckPermissionResponse>, Status> {
        let req = request.into_inner();

        let service_name = if req.service_name.is_empty() {
            None
        } else {
            Some(req.service_name.as_str())
        };

        let allowed = self
            .service
            .check_permission(req.user_id, &req.permission_key, service_name)
            .await
            .map_err(|e| Status::new(e.to_grpc_code().into(), e.to_string()))?;

        Ok(Response::new(CheckPermissionResponse { allowed }))
    }

    async fn get_user(
        &self,
        request: Request<GetUserRequest>,
    ) -> Result<Response<GetUserResponse>, Status> {
        let req = request.into_inner();

        let user = self
            .service
            .get_user(req.user_id)
            .await
            .map_err(|e| Status::new(e.to_grpc_code().into(), e.to_string()))?;

        Ok(Response::new(GetUserResponse {
            user: Some(user_to_proto(user)),
        }))
    }

    async fn list_user_roles(
        &self,
        request: Request<ListUserRolesRequest>,
    ) -> Result<Response<ListUserRolesResponse>, Status> {
        let req = request.into_inner();

        let roles = self
            .service
            .list_user_roles(req.user_id)
            .await
            .map_err(|e| Status::new(e.to_grpc_code().into(), e.to_string()))?;

        Ok(Response::new(ListUserRolesResponse {
            roles: roles.into_iter().map(role_to_proto).collect(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Role, User, UserStatus};
    use crate::infrastructure::{
        InMemoryPermissionRepository, InMemoryRoleRepository, InMemoryTokenRepository,
        InMemoryUserRepository,
    };
    use crate::application::AuthService;
    use std::time::Duration;

    // ========================================
    // Helper Functions
    // ========================================

    type TestUserRepo = InMemoryUserRepository;
    type TestRoleRepo = InMemoryRoleRepository;
    type TestPermRepo = InMemoryPermissionRepository;
    type TestTokenRepo = InMemoryTokenRepository;
    type TestAuthService = AuthService<TestUserRepo, TestRoleRepo, TestPermRepo, TestTokenRepo>;
    type TestGrpcService = GrpcAuthService<TestUserRepo, TestRoleRepo, TestPermRepo, TestTokenRepo>;

    fn create_grpc_service() -> (TestGrpcService, Arc<TestAuthService>) {
        let user_repo = Arc::new(InMemoryUserRepository::new());
        let role_repo = Arc::new(InMemoryRoleRepository::new());
        let perm_repo = Arc::new(InMemoryPermissionRepository::new());
        let token_repo = Arc::new(InMemoryTokenRepository::new());

        let auth_service = AuthService::new(
            user_repo,
            role_repo,
            perm_repo,
            token_repo,
            "k1s0-test",
            "secret123",
        );
        let auth_service = Arc::new(auth_service);
        let grpc_service = GrpcAuthService::new(Arc::clone(&auth_service));
        (grpc_service, auth_service)
    }

    // ========================================
    // Time Conversion Tests
    // ========================================

    #[test]
    fn test_system_time_to_rfc3339_unix_epoch() {
        let time = SystemTime::UNIX_EPOCH;
        let result = system_time_to_rfc3339(time);
        assert_eq!(result, "1970-01-01T00:00:00Z");
    }

    #[test]
    fn test_system_time_to_rfc3339_specific_time() {
        // 2024-01-01 00:00:00 UTC = 1704067200 seconds since epoch
        let time = SystemTime::UNIX_EPOCH + Duration::from_secs(1704067200);
        let result = system_time_to_rfc3339(time);
        assert_eq!(result, "2024-01-01T00:00:00Z");
    }

    #[test]
    fn test_option_system_time_to_rfc3339_none() {
        let result = option_system_time_to_rfc3339(None);
        assert_eq!(result, "");
    }

    #[test]
    fn test_option_system_time_to_rfc3339_some() {
        let time = SystemTime::UNIX_EPOCH;
        let result = option_system_time_to_rfc3339(Some(time));
        assert_eq!(result, "1970-01-01T00:00:00Z");
    }

    // ========================================
    // User to Proto Conversion Tests
    // ========================================

    #[test]
    fn test_user_to_proto_basic() {
        let user = User::new(1, "testuser", "test@example.com", "Test User", "hash123");
        let proto = user_to_proto(user);

        assert_eq!(proto.user_id, 1);
        assert_eq!(proto.login_id, "testuser");
        assert_eq!(proto.email, "test@example.com");
        assert_eq!(proto.display_name, "Test User");
        assert_eq!(proto.status, 1); // Active = 1
    }

    #[test]
    fn test_user_to_proto_inactive_status() {
        let user = User::new(1, "testuser", "test@example.com", "Test User", "hash123")
            .with_status(UserStatus::Inactive);
        let proto = user_to_proto(user);

        assert_eq!(proto.status, 0); // Inactive = 0
    }

    #[test]
    fn test_user_to_proto_locked_status() {
        let user = User::new(1, "testuser", "test@example.com", "Test User", "hash123")
            .with_status(UserStatus::Locked);
        let proto = user_to_proto(user);

        assert_eq!(proto.status, 2); // Locked = 2
    }

    #[test]
    fn test_user_to_proto_timestamps_format() {
        let user = User::new(1, "testuser", "test@example.com", "Test User", "hash123");
        let proto = user_to_proto(user);

        // Timestamps should be in RFC3339 format
        assert!(proto.created_at.contains("T"));
        assert!(proto.created_at.ends_with("Z"));
        assert!(proto.updated_at.contains("T"));
        assert!(proto.updated_at.ends_with("Z"));
    }

    #[test]
    fn test_user_to_proto_last_login_empty_when_none() {
        let user = User::new(1, "testuser", "test@example.com", "Test User", "hash123");
        let proto = user_to_proto(user);

        // last_login_at should be empty string when None
        assert_eq!(proto.last_login_at, "");
    }

    // ========================================
    // Role to Proto Conversion Tests
    // ========================================

    #[test]
    fn test_role_to_proto() {
        let role = Role::new(1, "admin", "Administrator role");
        let proto = role_to_proto(role);

        assert_eq!(proto.role_id, 1);
        assert_eq!(proto.role_name, "admin");
        assert_eq!(proto.description, "Administrator role");
    }

    #[test]
    fn test_role_to_proto_empty_description() {
        let role = Role::new(1, "user", "");
        let proto = role_to_proto(role);

        assert_eq!(proto.description, "");
    }

    // ========================================
    // gRPC Service Creation Tests
    // ========================================

    #[test]
    fn test_grpc_service_creation() {
        let (_grpc, _auth) = create_grpc_service();
        // Service should be created without panicking
    }

    // ========================================
    // Authenticate Tests
    // ========================================

    #[tokio::test]
    async fn test_authenticate_success() {
        let (grpc_service, auth_service) = create_grpc_service();

        // Add a test user
        let user = User::new(1, "testuser", "test@example.com", "Test User", "hash:password123");
        auth_service.user_repo.save(&user).await.unwrap();

        let request = Request::new(AuthenticateRequest {
            login_id: "testuser".to_string(),
            password: "password123".to_string(),
        });

        let response = grpc_service.authenticate(request).await;
        assert!(response.is_ok());

        let auth_response = response.unwrap().into_inner();
        assert!(!auth_response.access_token.is_empty());
        assert!(!auth_response.refresh_token.is_empty());
        assert_eq!(auth_response.token_type, "Bearer");
        assert!(auth_response.expires_in > 0);
    }

    #[tokio::test]
    async fn test_authenticate_invalid_credentials() {
        let (grpc_service, auth_service) = create_grpc_service();

        let user = User::new(1, "testuser", "test@example.com", "Test User", "hash:password123");
        auth_service.user_repo.save(&user).await.unwrap();

        let request = Request::new(AuthenticateRequest {
            login_id: "testuser".to_string(),
            password: "wrongpassword".to_string(),
        });

        let response = grpc_service.authenticate(request).await;
        assert!(response.is_err());

        let status = response.unwrap_err();
        assert_eq!(status.code(), tonic::Code::Unauthenticated);
    }

    #[tokio::test]
    async fn test_authenticate_user_not_found() {
        let (grpc_service, _auth_service) = create_grpc_service();

        let request = Request::new(AuthenticateRequest {
            login_id: "nonexistent".to_string(),
            password: "password".to_string(),
        });

        let response = grpc_service.authenticate(request).await;
        assert!(response.is_err());
    }

    // ========================================
    // Refresh Token Tests
    // ========================================

    #[tokio::test]
    async fn test_refresh_token_success() {
        let (grpc_service, auth_service) = create_grpc_service();

        let user = User::new(1, "testuser", "test@example.com", "Test User", "hash:password123");
        auth_service.user_repo.save(&user).await.unwrap();

        // First authenticate to get a refresh token
        let auth_request = Request::new(AuthenticateRequest {
            login_id: "testuser".to_string(),
            password: "password123".to_string(),
        });
        let auth_response = grpc_service.authenticate(auth_request).await.unwrap().into_inner();

        // Now refresh the token
        let refresh_request = Request::new(RefreshTokenRequest {
            refresh_token: auth_response.refresh_token,
        });

        let refresh_response = grpc_service.refresh_token(refresh_request).await;
        assert!(refresh_response.is_ok());

        let new_token = refresh_response.unwrap().into_inner();
        assert!(!new_token.access_token.is_empty());
        assert!(!new_token.refresh_token.is_empty());
    }

    #[tokio::test]
    async fn test_refresh_token_invalid() {
        let (grpc_service, _auth_service) = create_grpc_service();

        let request = Request::new(RefreshTokenRequest {
            refresh_token: "invalid_token".to_string(),
        });

        let response = grpc_service.refresh_token(request).await;
        assert!(response.is_err());
    }

    // ========================================
    // Check Permission Tests
    // ========================================

    #[tokio::test]
    async fn test_check_permission_allowed() {
        let (grpc_service, auth_service) = create_grpc_service();

        let user = User::new(1, "admin", "admin@example.com", "Admin", "hash:password");
        auth_service.user_repo.save(&user).await.unwrap();

        auth_service.permission_repo.add_permission(1, "user:read", None);

        let request = Request::new(CheckPermissionRequest {
            user_id: 1,
            permission_key: "user:read".to_string(),
            service_name: String::new(),
        });

        let response = grpc_service.check_permission(request).await;
        assert!(response.is_ok());
        assert!(response.unwrap().into_inner().allowed);
    }

    #[tokio::test]
    async fn test_check_permission_denied() {
        let (grpc_service, auth_service) = create_grpc_service();

        let user = User::new(1, "user", "user@example.com", "User", "hash:password");
        auth_service.user_repo.save(&user).await.unwrap();

        let request = Request::new(CheckPermissionRequest {
            user_id: 1,
            permission_key: "admin:all".to_string(),
            service_name: String::new(),
        });

        let response = grpc_service.check_permission(request).await;
        assert!(response.is_ok());
        assert!(!response.unwrap().into_inner().allowed);
    }

    #[tokio::test]
    async fn test_check_permission_with_service_name() {
        let (grpc_service, auth_service) = create_grpc_service();

        let user = User::new(1, "user", "user@example.com", "User", "hash:password");
        auth_service.user_repo.save(&user).await.unwrap();

        auth_service.permission_repo.add_permission(1, "resource:read", Some("resource-svc"));

        let request = Request::new(CheckPermissionRequest {
            user_id: 1,
            permission_key: "resource:read".to_string(),
            service_name: "resource-svc".to_string(),
        });

        let response = grpc_service.check_permission(request).await;
        assert!(response.is_ok());
        assert!(response.unwrap().into_inner().allowed);
    }

    #[tokio::test]
    async fn test_check_permission_user_not_found() {
        let (grpc_service, _auth_service) = create_grpc_service();

        let request = Request::new(CheckPermissionRequest {
            user_id: 999,
            permission_key: "user:read".to_string(),
            service_name: String::new(),
        });

        let response = grpc_service.check_permission(request).await;
        assert!(response.is_err());
    }

    // ========================================
    // Get User Tests
    // ========================================

    #[tokio::test]
    async fn test_get_user_success() {
        let (grpc_service, auth_service) = create_grpc_service();

        let user = User::new(1, "testuser", "test@example.com", "Test User", "hash:password");
        auth_service.user_repo.save(&user).await.unwrap();

        let request = Request::new(GetUserRequest { user_id: 1 });

        let response = grpc_service.get_user(request).await;
        assert!(response.is_ok());

        let user_response = response.unwrap().into_inner();
        assert!(user_response.user.is_some());

        let proto_user = user_response.user.unwrap();
        assert_eq!(proto_user.user_id, 1);
        assert_eq!(proto_user.login_id, "testuser");
        assert_eq!(proto_user.email, "test@example.com");
    }

    #[tokio::test]
    async fn test_get_user_not_found() {
        let (grpc_service, _auth_service) = create_grpc_service();

        let request = Request::new(GetUserRequest { user_id: 999 });

        let response = grpc_service.get_user(request).await;
        assert!(response.is_err());

        let status = response.unwrap_err();
        assert_eq!(status.code(), tonic::Code::NotFound);
    }

    // ========================================
    // List User Roles Tests
    // ========================================

    #[tokio::test]
    async fn test_list_user_roles_success() {
        let (grpc_service, auth_service) = create_grpc_service();

        let user = User::new(1, "testuser", "test@example.com", "Test User", "hash:password");
        auth_service.user_repo.save(&user).await.unwrap();

        let admin_role = Role::new(1, "admin", "Administrator");
        let user_role = Role::new(2, "user", "Normal User");
        auth_service.role_repo.add_role(admin_role);
        auth_service.role_repo.add_role(user_role);
        auth_service.role_repo.assign_role(1, 1).await.unwrap();
        auth_service.role_repo.assign_role(1, 2).await.unwrap();

        let request = Request::new(ListUserRolesRequest { user_id: 1 });

        let response = grpc_service.list_user_roles(request).await;
        assert!(response.is_ok());

        let roles_response = response.unwrap().into_inner();
        assert_eq!(roles_response.roles.len(), 2);
    }

    #[tokio::test]
    async fn test_list_user_roles_empty() {
        let (grpc_service, auth_service) = create_grpc_service();

        let user = User::new(1, "testuser", "test@example.com", "Test User", "hash:password");
        auth_service.user_repo.save(&user).await.unwrap();

        let request = Request::new(ListUserRolesRequest { user_id: 1 });

        let response = grpc_service.list_user_roles(request).await;
        assert!(response.is_ok());

        let roles_response = response.unwrap().into_inner();
        assert!(roles_response.roles.is_empty());
    }

    #[tokio::test]
    async fn test_list_user_roles_user_not_found() {
        let (grpc_service, _auth_service) = create_grpc_service();

        let request = Request::new(ListUserRolesRequest { user_id: 999 });

        let response = grpc_service.list_user_roles(request).await;
        assert!(response.is_err());
    }
}
