use std::collections::HashMap;
use std::sync::Arc;

use k1s0_auth_server::adapter::grpc::auth_grpc::{
    AuthGrpcService, CheckPermissionRequest, GetUserRequest, GetUserRolesRequest, ListUsersRequest,
    PbPagination, ValidateTokenRequest,
};
use k1s0_auth_server::domain::entity::claims::{Claims, RealmAccess};
use k1s0_auth_server::domain::entity::user::{Pagination, Role, User, UserListResult, UserRoles};
use k1s0_auth_server::domain::repository::UserRepository;
use k1s0_auth_server::infrastructure::TokenVerifier;
use k1s0_auth_server::usecase::get_user::GetUserUseCase;
use k1s0_auth_server::usecase::get_user_roles::GetUserRolesUseCase;
use k1s0_auth_server::usecase::list_users::ListUsersUseCase;
use k1s0_auth_server::usecase::validate_token::ValidateTokenUseCase;

// --- Test doubles ---

struct GrpcTestTokenVerifier {
    should_succeed: bool,
}

#[async_trait::async_trait]
impl TokenVerifier for GrpcTestTokenVerifier {
    async fn verify_token(&self, _token: &str) -> anyhow::Result<Claims> {
        if self.should_succeed {
            Ok(Claims {
                sub: "grpc-test-user-1".to_string(),
                iss: "test-issuer".to_string(),
                aud: "test-audience".to_string(),
                exp: chrono::Utc::now().timestamp() + 3600,
                iat: chrono::Utc::now().timestamp(),
                jti: "grpc-token-uuid".to_string(),
                typ: "Bearer".to_string(),
                azp: "react-spa".to_string(),
                scope: "openid profile email".to_string(),
                preferred_username: "grpc.test.user".to_string(),
                email: "grpc.test@example.com".to_string(),
                realm_access: RealmAccess {
                    roles: vec!["user".to_string(), "sys_auditor".to_string()],
                },
                resource_access: HashMap::new(),
                tier_access: vec!["system".to_string()],
            })
        } else {
            anyhow::bail!("token verification failed")
        }
    }
}

struct GrpcTestUserRepository;

#[async_trait::async_trait]
impl UserRepository for GrpcTestUserRepository {
    async fn find_by_id(&self, user_id: &str) -> anyhow::Result<User> {
        if user_id == "grpc-user-1" {
            Ok(User {
                id: "grpc-user-1".to_string(),
                username: "grpc.test.user".to_string(),
                email: "grpc.test@example.com".to_string(),
                first_name: "Grpc".to_string(),
                last_name: "Test".to_string(),
                enabled: true,
                email_verified: true,
                created_at: chrono::Utc::now(),
                attributes: HashMap::new(),
            })
        } else {
            anyhow::bail!("user not found: {}", user_id)
        }
    }

    async fn list(
        &self,
        page: i32,
        page_size: i32,
        _search: Option<String>,
        _enabled: Option<bool>,
    ) -> anyhow::Result<UserListResult> {
        Ok(UserListResult {
            users: vec![
                User {
                    id: "grpc-user-1".to_string(),
                    username: "grpc.test.user".to_string(),
                    email: "grpc.test@example.com".to_string(),
                    first_name: "Grpc".to_string(),
                    last_name: "Test".to_string(),
                    enabled: true,
                    email_verified: true,
                    created_at: chrono::Utc::now(),
                    attributes: HashMap::new(),
                },
                User {
                    id: "grpc-user-2".to_string(),
                    username: "grpc.test.user2".to_string(),
                    email: "grpc.test2@example.com".to_string(),
                    first_name: "Grpc2".to_string(),
                    last_name: "Test2".to_string(),
                    enabled: true,
                    email_verified: false,
                    created_at: chrono::Utc::now(),
                    attributes: HashMap::new(),
                },
            ],
            pagination: Pagination {
                total_count: 2,
                page,
                page_size,
                has_next: false,
            },
        })
    }

    async fn get_roles(&self, user_id: &str) -> anyhow::Result<UserRoles> {
        if user_id == "grpc-user-1" {
            Ok(UserRoles {
                user_id: "grpc-user-1".to_string(),
                realm_roles: vec![
                    Role {
                        id: "role-1".to_string(),
                        name: "user".to_string(),
                        description: "General user".to_string(),
                    },
                    Role {
                        id: "role-2".to_string(),
                        name: "sys_auditor".to_string(),
                        description: "Auditor".to_string(),
                    },
                ],
                client_roles: HashMap::from([(
                    "order-service".to_string(),
                    vec![Role {
                        id: "role-3".to_string(),
                        name: "read".to_string(),
                        description: "Read access".to_string(),
                    }],
                )]),
            })
        } else {
            anyhow::bail!("user not found: {}", user_id)
        }
    }
}

fn make_grpc_service(token_success: bool) -> AuthGrpcService {
    let verifier: Arc<dyn TokenVerifier> = Arc::new(GrpcTestTokenVerifier {
        should_succeed: token_success,
    });
    let user_repo: Arc<dyn UserRepository> = Arc::new(GrpcTestUserRepository);

    let validate_uc = Arc::new(ValidateTokenUseCase::new(
        verifier,
        "test-issuer".to_string(),
        "test-audience".to_string(),
    ));
    let get_user_uc = Arc::new(GetUserUseCase::new(user_repo.clone()));
    let get_user_roles_uc = Arc::new(GetUserRolesUseCase::new(user_repo.clone()));
    let list_users_uc = Arc::new(ListUsersUseCase::new(user_repo));

    AuthGrpcService::new(validate_uc, get_user_uc, get_user_roles_uc, list_users_uc)
}

// --- gRPC Integration Tests ---

#[tokio::test]
async fn test_validate_token_grpc_success() {
    let svc = make_grpc_service(true);

    let req = ValidateTokenRequest {
        token: "valid-grpc-token".to_string(),
    };
    let resp = svc.validate_token(req).await.unwrap();

    assert!(resp.valid);
    let claims = resp.claims.unwrap();
    assert_eq!(claims.sub, "grpc-test-user-1");
    assert_eq!(claims.preferred_username, "grpc.test.user");
    assert_eq!(claims.email, "grpc.test@example.com");
    assert!(resp.error_message.is_empty());
}

#[tokio::test]
async fn test_validate_token_grpc_invalid() {
    let svc = make_grpc_service(false);

    let req = ValidateTokenRequest {
        token: "invalid-grpc-token".to_string(),
    };
    let resp = svc.validate_token(req).await.unwrap();

    assert!(!resp.valid);
    assert!(resp.claims.is_none());
    assert!(!resp.error_message.is_empty());
}

#[tokio::test]
async fn test_get_user_grpc_success() {
    let svc = make_grpc_service(true);

    let req = GetUserRequest {
        user_id: "grpc-user-1".to_string(),
    };
    let resp = svc.get_user(req).await.unwrap();

    let user = resp.user.unwrap();
    assert_eq!(user.id, "grpc-user-1");
    assert_eq!(user.username, "grpc.test.user");
    assert_eq!(user.email, "grpc.test@example.com");
    assert!(user.enabled);
}

#[tokio::test]
async fn test_get_user_grpc_not_found() {
    let svc = make_grpc_service(true);

    let req = GetUserRequest {
        user_id: "nonexistent-grpc-user".to_string(),
    };
    let result = svc.get_user(req).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        k1s0_auth_server::adapter::grpc::auth_grpc::GrpcError::NotFound(msg) => {
            assert!(msg.contains("not found"));
        }
        e => unreachable!("unexpected error in test: {:?}", e),
    }
}

#[tokio::test]
async fn test_list_users_grpc_success() {
    let svc = make_grpc_service(true);

    let req = ListUsersRequest {
        pagination: None,
        search: String::new(),
        enabled: None,
    };
    let resp = svc.list_users(req).await.unwrap();

    assert_eq!(resp.users.len(), 2);
    assert_eq!(resp.users[0].id, "grpc-user-1");
    assert_eq!(resp.users[1].id, "grpc-user-2");

    let pagination = resp.pagination.unwrap();
    assert_eq!(pagination.total_count, 2);
    assert!(!pagination.has_next);
}

#[tokio::test]
async fn test_list_users_grpc_with_pagination() {
    let svc = make_grpc_service(true);

    let req = ListUsersRequest {
        pagination: Some(PbPagination {
            page: 1,
            page_size: 10,
        }),
        search: String::new(),
        enabled: None,
    };
    let resp = svc.list_users(req).await.unwrap();

    assert_eq!(resp.users.len(), 2);
    let pagination = resp.pagination.unwrap();
    assert_eq!(pagination.page, 1);
    assert_eq!(pagination.page_size, 10);
    assert_eq!(pagination.total_count, 2);
}

#[tokio::test]
async fn test_check_permission_grpc_success() {
    let svc = make_grpc_service(true);

    let req = CheckPermissionRequest {
        user_id: "grpc-user-1".to_string(),
        permission: "admin".to_string(),
        resource: "users".to_string(),
        roles: vec!["sys_admin".to_string()],
    };
    let resp = svc.check_permission(req).await.unwrap();

    assert!(resp.allowed);
    assert!(resp.reason.is_empty());
}

#[tokio::test]
async fn test_check_permission_grpc_denied() {
    let svc = make_grpc_service(true);

    let req = CheckPermissionRequest {
        user_id: "grpc-user-1".to_string(),
        permission: "admin".to_string(),
        resource: "users".to_string(),
        roles: vec!["user".to_string()],
    };
    let resp = svc.check_permission(req).await.unwrap();

    assert!(!resp.allowed);
    assert!(resp.reason.contains("insufficient permissions"));
}

#[tokio::test]
async fn test_get_user_roles_grpc_success() {
    let svc = make_grpc_service(true);

    let req = GetUserRolesRequest {
        user_id: "grpc-user-1".to_string(),
    };
    let resp = svc.get_user_roles(req).await.unwrap();

    assert_eq!(resp.user_id, "grpc-user-1");
    assert_eq!(resp.realm_roles.len(), 2);
    assert_eq!(resp.realm_roles[0].name, "user");
    assert_eq!(resp.realm_roles[1].name, "sys_auditor");
    assert_eq!(resp.client_roles.len(), 1);
    assert!(resp.client_roles.contains_key("order-service"));
    assert_eq!(resp.client_roles["order-service"].roles.len(), 1);
    assert_eq!(resp.client_roles["order-service"].roles[0].name, "read");
}
