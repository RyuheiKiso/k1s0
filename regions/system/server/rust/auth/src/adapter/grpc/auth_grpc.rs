use std::collections::HashMap;
use std::sync::Arc;

use crate::domain::entity::claims::Claims;
use crate::domain::entity::user::User;
use crate::proto::k1s0::system::auth::v1::{
    self as proto_auth, CheckPermissionRequest, CheckPermissionResponse, ClientRoles,
    GetUserRequest, GetUserResponse, GetUserRolesRequest, GetUserRolesResponse, ListUsersRequest,
    ListUsersResponse, RealmAccess, Role as ProtoRole, RoleList, StringList, TokenClaims,
    ValidateTokenRequest, ValidateTokenResponse,
};
use crate::proto::k1s0::system::common::v1::{PaginationResult, Timestamp};
use crate::usecase::check_permission::{CheckPermissionInput, CheckPermissionUseCase};
use crate::usecase::get_user::{GetUserError, GetUserUseCase};
use crate::usecase::get_user_roles::{GetUserRolesError, GetUserRolesUseCase};
use crate::usecase::list_users::{ListUsersParams, ListUsersUseCase};
use crate::usecase::validate_token::ValidateTokenUseCase;

// --- gRPC Error ---

#[derive(Debug, thiserror::Error)]
pub enum GrpcError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("unauthenticated: {0}")]
    Unauthenticated(String),

    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    #[error("internal: {0}")]
    Internal(String),
}

// --- AuthGrpcService ---

pub struct AuthGrpcService {
    validate_token_uc: Arc<ValidateTokenUseCase>,
    get_user_uc: Arc<GetUserUseCase>,
    get_user_roles_uc: Arc<GetUserRolesUseCase>,
    list_users_uc: Arc<ListUsersUseCase>,
    check_permission_uc: Arc<CheckPermissionUseCase>,
}

impl AuthGrpcService {
    pub fn new(
        validate_token_uc: Arc<ValidateTokenUseCase>,
        get_user_uc: Arc<GetUserUseCase>,
        get_user_roles_uc: Arc<GetUserRolesUseCase>,
        list_users_uc: Arc<ListUsersUseCase>,
        check_permission_uc: Arc<CheckPermissionUseCase>,
    ) -> Self {
        Self {
            validate_token_uc,
            get_user_uc,
            get_user_roles_uc,
            list_users_uc,
            check_permission_uc,
        }
    }

    /// JWT トークン検証。
    pub async fn validate_token(
        &self,
        req: ValidateTokenRequest,
    ) -> Result<ValidateTokenResponse, GrpcError> {
        match self.validate_token_uc.execute(&req.token).await {
            Ok(claims) => Ok(ValidateTokenResponse {
                valid: true,
                claims: Some(domain_claims_to_proto(&claims)),
                error_message: String::new(),
            }),
            Err(e) => Ok(ValidateTokenResponse {
                valid: false,
                claims: None,
                error_message: e.to_string(),
            }),
        }
    }

    /// ユーザー情報取得。
    pub async fn get_user(&self, req: GetUserRequest) -> Result<GetUserResponse, GrpcError> {
        match self.get_user_uc.execute(&req.user_id).await {
            Ok(user) => Ok(GetUserResponse {
                user: Some(domain_user_to_proto(&user)),
            }),
            Err(GetUserError::NotFound(id)) => {
                Err(GrpcError::NotFound(format!("user not found: {}", id)))
            }
            Err(e) => Err(GrpcError::Internal(e.to_string())),
        }
    }

    /// ユーザー一覧取得。
    pub async fn list_users(&self, req: ListUsersRequest) -> Result<ListUsersResponse, GrpcError> {
        let params = ListUsersParams {
            page: req.pagination.as_ref().map(|p| p.page),
            page_size: req.pagination.as_ref().map(|p| p.page_size),
            search: if req.search.is_empty() {
                None
            } else {
                Some(req.search)
            },
            enabled: req.enabled,
        };

        match self.list_users_uc.execute(&params).await {
            Ok(result) => {
                let proto_users: Vec<proto_auth::User> =
                    result.users.iter().map(domain_user_to_proto).collect();

                Ok(ListUsersResponse {
                    users: proto_users,
                    pagination: Some(PaginationResult {
                        total_count: result.pagination.total_count,
                        page: result.pagination.page,
                        page_size: result.pagination.page_size,
                        has_next: result.pagination.has_next,
                    }),
                })
            }
            Err(e) => Err(GrpcError::Internal(e.to_string())),
        }
    }

    /// ユーザーロール取得。
    pub async fn get_user_roles(
        &self,
        req: GetUserRolesRequest,
    ) -> Result<GetUserRolesResponse, GrpcError> {
        match self.get_user_roles_uc.execute(&req.user_id).await {
            Ok(user_roles) => {
                let proto_realm_roles: Vec<ProtoRole> = user_roles
                    .realm_roles
                    .iter()
                    .map(|r| ProtoRole {
                        id: r.id.clone(),
                        name: r.name.clone(),
                        description: r.description.clone(),
                    })
                    .collect();

                let mut proto_client_roles = HashMap::new();
                for (client_id, roles) in &user_roles.client_roles {
                    let pb_roles: Vec<ProtoRole> = roles
                        .iter()
                        .map(|r| ProtoRole {
                            id: r.id.clone(),
                            name: r.name.clone(),
                            description: r.description.clone(),
                        })
                        .collect();
                    proto_client_roles.insert(client_id.clone(), RoleList { roles: pb_roles });
                }

                Ok(GetUserRolesResponse {
                    user_id: req.user_id,
                    realm_roles: proto_realm_roles,
                    client_roles: proto_client_roles,
                })
            }
            Err(GetUserRolesError::NotFound(id)) => {
                Err(GrpcError::NotFound(format!("user not found: {}", id)))
            }
            Err(e) => Err(GrpcError::Internal(e.to_string())),
        }
    }

    /// パーミッション確認。
    pub async fn check_permission(
        &self,
        req: CheckPermissionRequest,
    ) -> Result<CheckPermissionResponse, GrpcError> {
        let input = CheckPermissionInput {
            user_id: req.user_id.and_then(|id| {
                let trimmed = id.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed.to_string())
                }
            }),
            roles: req.roles,
            permission: req.permission,
            resource: req.resource,
        };
        let result = self.check_permission_uc.execute(&input).await;
        Ok(CheckPermissionResponse {
            allowed: result.allowed,
            reason: result.reason,
        })
    }
}

// --- 変換ヘルパー ---

fn domain_claims_to_proto(c: &Claims) -> TokenClaims {
    let mut resource_access = HashMap::new();
    for (k, v) in &c.resource_access {
        resource_access.insert(
            k.clone(),
            ClientRoles {
                roles: v.roles.clone(),
            },
        );
    }

    TokenClaims {
        sub: c.sub.clone(),
        iss: c.iss.clone(),
        // proto の TokenClaims.aud は repeated string 型（CRIT-006 対応）。
        // JWT spec の audience は配列なので Vec<String> をそのまま渡す。
        aud: c.aud.clone(),
        exp: c.exp,
        iat: c.iat,
        jti: c.jti.clone(),
        typ: Some(c.typ.clone()),
        azp: Some(c.azp.clone()),
        preferred_username: c.preferred_username.clone(),
        email: c.email.clone(),
        realm_access: Some(RealmAccess {
            roles: c.realm_access.roles.clone(),
        }),
        resource_access,
        tier_access: c.tier_access.clone(),
        scope: c.scope.clone(),
    }
}

fn domain_user_to_proto(u: &User) -> proto_auth::User {
    let mut attributes = HashMap::new();
    for (k, v) in &u.attributes {
        attributes.insert(k.clone(), StringList { values: v.clone() });
    }

    proto_auth::User {
        id: u.id.clone(),
        username: u.username.clone(),
        email: u.email.clone(),
        first_name: u.first_name.clone(),
        last_name: u.last_name.clone(),
        enabled: u.enabled,
        email_verified: u.email_verified,
        created_at: Some(Timestamp {
            seconds: u.created_at.timestamp(),
            nanos: u.created_at.timestamp_subsec_nanos() as i32,
        }),
        attributes,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::entity::claims::RealmAccess as DomainRealmAccess;
    use crate::domain::entity::user::{Pagination, Role, UserListResult, UserRoles};
    use crate::domain::repository::user_repository::MockUserRepository;
    use crate::infrastructure::MockTokenVerifier;
    use std::collections::HashMap;

    fn make_valid_claims() -> Claims {
        Claims {
            sub: "user-uuid-1234".to_string(),
            iss: "https://auth.k1s0.internal.example.com/realms/k1s0".to_string(),
            // aud を Vec<String> で設定する（複数 audience 対応）
            aud: vec!["k1s0-api".to_string()],
            exp: chrono::Utc::now().timestamp() + 3600,
            iat: chrono::Utc::now().timestamp(),
            jti: "token-uuid-5678".to_string(),
            typ: "Bearer".to_string(),
            azp: "react-spa".to_string(),
            scope: "openid profile email".to_string(),
            preferred_username: "taro.yamada".to_string(),
            email: "taro.yamada@example.com".to_string(),
            realm_access: DomainRealmAccess {
                roles: vec!["user".to_string(), "sys_auditor".to_string()],
            },
            resource_access: HashMap::new(),
            tier_access: vec!["system".to_string()],
        }
    }

    fn make_auth_service(
        verifier: MockTokenVerifier,
        user_repo: MockUserRepository,
    ) -> AuthGrpcService {
        use crate::usecase::check_permission::CheckPermissionUseCase;
        use crate::usecase::get_user_roles::GetUserRolesUseCase;
        let validate_uc = Arc::new(ValidateTokenUseCase::new(
            Arc::new(verifier),
            "https://auth.k1s0.internal.example.com/realms/k1s0".to_string(),
            "k1s0-api".to_string(),
        ));
        let user_repo = Arc::new(user_repo);
        let get_user_uc = Arc::new(GetUserUseCase::new(user_repo.clone()));
        let get_user_roles_uc = Arc::new(GetUserRolesUseCase::new(user_repo.clone()));
        let list_users_uc = Arc::new(ListUsersUseCase::new(user_repo.clone()));
        let check_permission_uc =
            Arc::new(CheckPermissionUseCase::with_user_repo(user_repo.clone()));

        AuthGrpcService::new(
            validate_uc,
            get_user_uc,
            get_user_roles_uc,
            list_users_uc,
            check_permission_uc,
        )
    }

    #[tokio::test]
    async fn test_validate_token_success() {
        let mut mock_verifier = MockTokenVerifier::new();
        let claims = make_valid_claims();
        let return_claims = claims.clone();

        mock_verifier
            .expect_verify_token()
            .returning(move |_| Ok(return_claims.clone()));

        let svc = make_auth_service(mock_verifier, MockUserRepository::new());

        let req = ValidateTokenRequest {
            token: "valid-token".to_string(),
        };
        let resp = svc
            .validate_token(req)
            .await
            .expect("validate_token should succeed");

        assert!(resp.valid);
        let proto_claims = resp.claims.expect("claims should be present");
        assert_eq!(proto_claims.sub, "user-uuid-1234");
        assert_eq!(proto_claims.preferred_username, "taro.yamada");
        assert_eq!(proto_claims.email, "taro.yamada@example.com");
        assert_eq!(proto_claims.scope, "openid profile email");
    }

    #[tokio::test]
    async fn test_validate_token_invalid_token() {
        let mut mock_verifier = MockTokenVerifier::new();
        mock_verifier
            .expect_verify_token()
            .returning(|_| Err(anyhow::anyhow!("invalid signature")));

        let svc = make_auth_service(mock_verifier, MockUserRepository::new());

        let req = ValidateTokenRequest {
            token: "invalid-token".to_string(),
        };
        let resp = svc
            .validate_token(req)
            .await
            .expect("validate_token should succeed");

        assert!(!resp.valid);
        assert!(resp.claims.is_none());
        assert!(resp.error_message.contains("invalid"));
    }

    #[tokio::test]
    async fn test_validate_token_empty_token() {
        let mut mock_verifier = MockTokenVerifier::new();
        mock_verifier
            .expect_verify_token()
            .returning(|_| Err(anyhow::anyhow!("empty token")));

        let svc = make_auth_service(mock_verifier, MockUserRepository::new());

        let req = ValidateTokenRequest {
            token: "".to_string(),
        };
        let resp = svc
            .validate_token(req)
            .await
            .expect("validate_token should succeed");

        assert!(!resp.valid);
        assert!(!resp.error_message.is_empty());
    }

    #[tokio::test]
    async fn test_get_user_exists() {
        let mock_verifier = MockTokenVerifier::new();
        let mut mock_user_repo = MockUserRepository::new();

        mock_user_repo
            .expect_find_by_id()
            .withf(|id| id == "user-uuid-1234")
            .returning(|_| {
                Ok(User {
                    id: "user-uuid-1234".to_string(),
                    username: "taro.yamada".to_string(),
                    email: "taro.yamada@example.com".to_string(),
                    first_name: "Taro".to_string(),
                    last_name: "Yamada".to_string(),
                    enabled: true,
                    email_verified: true,
                    created_at: chrono::Utc::now(),
                    attributes: HashMap::new(),
                })
            });
        mock_user_repo.expect_list().returning(|_, _, _, _| {
            Ok(UserListResult {
                users: vec![],
                pagination: Pagination {
                    total_count: 0,
                    page: 1,
                    page_size: 20,
                    has_next: false,
                },
            })
        });

        let svc = make_auth_service(mock_verifier, mock_user_repo);

        let req = GetUserRequest {
            user_id: "user-uuid-1234".to_string(),
        };
        let resp = svc.get_user(req).await.expect("get_user should succeed");
        let user = resp.user.expect("user should be present");

        assert_eq!(user.id, "user-uuid-1234");
        assert_eq!(user.username, "taro.yamada");
        assert_eq!(user.email, "taro.yamada@example.com");
        assert!(user.enabled);
    }

    #[tokio::test]
    async fn test_get_user_not_found() {
        let mock_verifier = MockTokenVerifier::new();
        let mut mock_user_repo = MockUserRepository::new();

        mock_user_repo
            .expect_find_by_id()
            .returning(|_| Err(anyhow::anyhow!("user not found")));
        mock_user_repo.expect_list().returning(|_, _, _, _| {
            Ok(UserListResult {
                users: vec![],
                pagination: Pagination {
                    total_count: 0,
                    page: 1,
                    page_size: 20,
                    has_next: false,
                },
            })
        });

        let svc = make_auth_service(mock_verifier, mock_user_repo);

        let req = GetUserRequest {
            user_id: "nonexistent".to_string(),
        };
        let result = svc.get_user(req).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            GrpcError::NotFound(msg) => assert!(msg.contains("not found")),
            e => unreachable!("unexpected error in test: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_list_users_with_pagination() {
        use crate::proto::k1s0::system::common::v1::Pagination as ProtoPagination;

        let mock_verifier = MockTokenVerifier::new();
        let mut mock_user_repo = MockUserRepository::new();

        mock_user_repo
            .expect_find_by_id()
            .returning(|_| Err(anyhow::anyhow!("not used")));
        mock_user_repo
            .expect_list()
            .withf(|page, page_size, _, _| *page == 2 && *page_size == 10)
            .returning(|page, page_size, _, _| {
                Ok(UserListResult {
                    users: vec![User {
                        id: "user-1".to_string(),
                        username: "taro.yamada".to_string(),
                        email: "taro@example.com".to_string(),
                        first_name: "Taro".to_string(),
                        last_name: "Yamada".to_string(),
                        enabled: true,
                        email_verified: true,
                        created_at: chrono::Utc::now(),
                        attributes: HashMap::new(),
                    }],
                    pagination: Pagination {
                        total_count: 25,
                        page,
                        page_size,
                        has_next: true,
                    },
                })
            });

        let svc = make_auth_service(mock_verifier, mock_user_repo);

        let req = ListUsersRequest {
            pagination: Some(ProtoPagination {
                page: 2,
                page_size: 10,
            }),
            search: String::new(),
            enabled: None,
        };
        let resp = svc
            .list_users(req)
            .await
            .expect("list_users should succeed");

        assert_eq!(resp.users.len(), 1);
        assert_eq!(resp.users[0].id, "user-1");

        let pagination = resp.pagination.expect("pagination should be present");
        assert_eq!(pagination.total_count, 25);
        assert_eq!(pagination.page, 2);
        assert_eq!(pagination.page_size, 10);
        assert!(pagination.has_next);
    }

    #[tokio::test]
    async fn test_get_user_roles_exists() {
        let mock_verifier = MockTokenVerifier::new();
        let mut mock_user_repo = MockUserRepository::new();

        mock_user_repo
            .expect_find_by_id()
            .returning(|_| Err(anyhow::anyhow!("not used")));
        mock_user_repo.expect_list().returning(|_, _, _, _| {
            Ok(UserListResult {
                users: vec![],
                pagination: Pagination {
                    total_count: 0,
                    page: 1,
                    page_size: 20,
                    has_next: false,
                },
            })
        });
        mock_user_repo
            .expect_get_roles()
            .withf(|id| id == "user-uuid-1234")
            .returning(|_| {
                Ok(UserRoles {
                    user_id: "user-uuid-1234".to_string(),
                    realm_roles: vec![
                        Role {
                            id: "role-1".to_string(),
                            name: "user".to_string(),
                            description: "General user".to_string(),
                        },
                        Role {
                            id: "role-2".to_string(),
                            name: "sys_admin".to_string(),
                            description: "System admin".to_string(),
                        },
                    ],
                    client_roles: HashMap::from([(
                        "task-server".to_string(),
                        vec![Role {
                            id: "role-3".to_string(),
                            name: "read".to_string(),
                            description: "Read access".to_string(),
                        }],
                    )]),
                })
            });

        let svc = make_auth_service(mock_verifier, mock_user_repo);

        let req = GetUserRolesRequest {
            user_id: "user-uuid-1234".to_string(),
        };
        let resp = svc
            .get_user_roles(req)
            .await
            .expect("get_user_roles should succeed");

        assert_eq!(resp.user_id, "user-uuid-1234");
        assert_eq!(resp.realm_roles.len(), 2);
        assert_eq!(resp.realm_roles[0].name, "user");
        assert_eq!(resp.realm_roles[1].name, "sys_admin");
        assert_eq!(resp.client_roles["task-server"].roles.len(), 1);
        assert_eq!(resp.client_roles["task-server"].roles[0].name, "read");
    }

    #[tokio::test]
    async fn test_check_permission_allowed() {
        let mock_verifier = MockTokenVerifier::new();
        let mock_user_repo = MockUserRepository::new();
        let svc = make_auth_service(mock_verifier, mock_user_repo);

        let req = CheckPermissionRequest {
            user_id: None,
            permission: "admin".to_string(),
            resource: "users".to_string(),
            roles: vec!["sys_admin".to_string()],
        };
        let resp = svc
            .check_permission(req)
            .await
            .expect("check_permission should succeed");

        assert!(resp.allowed);
        assert!(resp.reason.is_empty());
    }

    #[tokio::test]
    async fn test_check_permission_denied() {
        let mock_verifier = MockTokenVerifier::new();
        let mock_user_repo = MockUserRepository::new();
        let svc = make_auth_service(mock_verifier, mock_user_repo);

        let req = CheckPermissionRequest {
            user_id: None,
            permission: "admin".to_string(),
            resource: "users".to_string(),
            roles: vec!["user".to_string()],
        };
        let resp = svc
            .check_permission(req)
            .await
            .expect("check_permission should succeed");

        assert!(!resp.allowed);
        assert!(resp.reason.contains("insufficient permissions"));
    }

    #[tokio::test]
    async fn test_check_permission_uses_user_id_roles() {
        let mock_verifier = MockTokenVerifier::new();
        let mut mock_user_repo = MockUserRepository::new();
        mock_user_repo
            .expect_get_roles()
            .withf(|id| id == "user-uuid-1234")
            .returning(|id| {
                Ok(UserRoles {
                    user_id: id.to_string(),
                    realm_roles: vec![Role {
                        id: "role-1".to_string(),
                        name: "sys_admin".to_string(),
                        description: String::new(),
                    }],
                    client_roles: HashMap::new(),
                })
            });
        let svc = make_auth_service(mock_verifier, mock_user_repo);

        let req = CheckPermissionRequest {
            user_id: Some("user-uuid-1234".to_string()),
            permission: "admin".to_string(),
            resource: "users".to_string(),
            roles: vec!["user".to_string()],
        };
        let resp = svc
            .check_permission(req)
            .await
            .expect("check_permission should succeed");

        assert!(resp.allowed);
    }
}
