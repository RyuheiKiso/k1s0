#![allow(clippy::unwrap_used)]
use std::collections::HashMap;
use std::sync::Arc;

use k1s0_auth_server::adapter::grpc::auth_grpc::{AuthGrpcService, GrpcError};
use k1s0_auth_server::domain::entity::claims::{Claims, RealmAccess};
use k1s0_auth_server::domain::entity::user::{Pagination, Role, User, UserListResult, UserRoles};
use k1s0_auth_server::domain::repository::UserRepository;
use k1s0_auth_server::infrastructure::TokenVerifier;
use k1s0_auth_server::proto::k1s0::system::auth::v1::{
    CheckPermissionRequest, GetUserRequest, GetUserRolesRequest, ListUsersRequest,
    ValidateTokenRequest,
};
use k1s0_auth_server::proto::k1s0::system::common::v1::Pagination as PbPagination;
use k1s0_auth_server::usecase::check_permission::CheckPermissionUseCase;
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
                    "task-server".to_string(),
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
    let list_users_uc = Arc::new(ListUsersUseCase::new(user_repo.clone()));
    let check_permission_uc = Arc::new(CheckPermissionUseCase::with_user_repo(user_repo));

    AuthGrpcService::new(
        validate_uc,
        get_user_uc,
        get_user_roles_uc,
        list_users_uc,
        check_permission_uc,
    )
}

// --- gRPC Integration Tests ---

// 有効なトークンで validate_token gRPC を呼び出すと成功レスポンスが返ることを検証する
#[tokio::test]
async fn test_validate_token_grpc_success() {
    let svc = make_grpc_service(true);

    let req = ValidateTokenRequest {
        token: "valid-grpc-token".to_string(),
    };
    // gRPC レスポンス取得: トークン検証 gRPC 呼び出しが失敗してはならない
    let resp = svc
        .validate_token(req)
        .await
        .expect("有効なトークンで validate_token gRPC 呼び出しが失敗しました");

    // 検証結果が有効であることを確認する
    assert!(
        resp.valid,
        "有効なトークンであれば valid フィールドが true でなければならない"
    );
    // クレームが返ることを確認する
    let claims = resp
        .claims
        .expect("有効なトークンであれば claims フィールドが Some でなければならない");
    assert_eq!(
        claims.sub, "grpc-test-user-1",
        "sub クレームが期待値 'grpc-test-user-1' と一致しなければならない"
    );
    assert_eq!(
        claims.preferred_username, "grpc.test.user",
        "preferred_username クレームが期待値 'grpc.test.user' と一致しなければならない"
    );
    assert_eq!(
        claims.email, "grpc.test@example.com",
        "email クレームが期待値 'grpc.test@example.com' と一致しなければならない"
    );
    assert!(
        resp.error_message.is_empty(),
        "有効なトークンであれば error_message は空でなければならない"
    );
}

// 無効なトークンで validate_token gRPC を呼び出すと無効レスポンスが返ることを検証する
#[tokio::test]
async fn test_validate_token_grpc_invalid() {
    let svc = make_grpc_service(false);

    let req = ValidateTokenRequest {
        token: "invalid-grpc-token".to_string(),
    };
    // gRPC レスポンス取得: 無効トークンでも gRPC 自体のエラーではなくレスポンスが返る
    let resp = svc
        .validate_token(req)
        .await
        .expect("無効なトークンでも validate_token gRPC 呼び出し自体は成功しなければならない");

    // 検証結果が無効であることを確認する
    assert!(
        !resp.valid,
        "無効なトークンであれば valid フィールドが false でなければならない"
    );
    assert!(
        resp.claims.is_none(),
        "無効なトークンであれば claims フィールドは None でなければならない"
    );
    assert!(
        !resp.error_message.is_empty(),
        "無効なトークンであれば error_message に理由が設定されていなければならない"
    );
}

// 存在するユーザー ID で get_user gRPC を呼び出すとユーザー情報が返ることを検証する
#[tokio::test]
async fn test_get_user_grpc_success() {
    let svc = make_grpc_service(true);

    let req = GetUserRequest {
        user_id: "grpc-user-1".to_string(),
    };
    // gRPC レスポンス取得: 存在するユーザーの取得に失敗してはならない
    let resp = svc
        .get_user(req)
        .await
        .expect("存在するユーザー ID で get_user gRPC 呼び出しが失敗しました");

    // ユーザー情報が返ることを確認する
    let user = resp
        .user
        .expect("存在するユーザーであれば user フィールドが Some でなければならない");
    assert_eq!(
        user.id, "grpc-user-1",
        "ユーザー ID が期待値 'grpc-user-1' と一致しなければならない"
    );
    assert_eq!(
        user.username, "grpc.test.user",
        "ユーザー名が期待値 'grpc.test.user' と一致しなければならない"
    );
    assert_eq!(
        user.email, "grpc.test@example.com",
        "メールアドレスが期待値 'grpc.test@example.com' と一致しなければならない"
    );
    assert!(
        user.enabled,
        "アクティブなユーザーであれば enabled フラグが true でなければならない"
    );
}

// 存在しないユーザー ID で get_user gRPC を呼び出すと NotFound エラーが返ることを検証する
#[tokio::test]
async fn test_get_user_grpc_not_found() {
    let svc = make_grpc_service(true);

    let req = GetUserRequest {
        user_id: "nonexistent-grpc-user".to_string(),
    };
    // 存在しないユーザー ID ではエラーが返ることを確認する
    let result = svc.get_user(req).await;

    assert!(
        result.is_err(),
        "存在しないユーザー ID の場合は Err が返らなければならない"
    );
    match result.unwrap_err() {
        GrpcError::NotFound(msg) => {
            assert!(
                msg.contains("not found"),
                "NotFound エラーメッセージに 'not found' が含まれていなければならない: 実際={}",
                msg
            );
        }
        e => unreachable!(
            "存在しないユーザー ID の場合は GrpcError::NotFound が返るべきだが、予期しないエラーが発生しました: {:?}",
            e
        ),
    }
}

// ページネーションなしで list_users gRPC を呼び出すと全ユーザーが返ることを検証する
#[tokio::test]
async fn test_list_users_grpc_success() {
    let svc = make_grpc_service(true);

    let req = ListUsersRequest {
        pagination: None,
        search: String::new(),
        enabled: None,
    };
    // gRPC レスポンス取得: ユーザー一覧の取得に失敗してはならない
    let resp = svc
        .list_users(req)
        .await
        .expect("list_users gRPC 呼び出しが失敗しました");

    assert_eq!(
        resp.users.len(),
        2,
        "テストリポジトリには 2 件のユーザーが存在するため、返却ユーザー数が 2 でなければならない"
    );
    assert_eq!(
        resp.users[0].id, "grpc-user-1",
        "1 件目のユーザー ID が 'grpc-user-1' でなければならない"
    );
    assert_eq!(
        resp.users[1].id, "grpc-user-2",
        "2 件目のユーザー ID が 'grpc-user-2' でなければならない"
    );

    // ページネーション情報が返ることを確認する
    let pagination = resp
        .pagination
        .expect("list_users レスポンスに pagination フィールドが存在しなければならない");
    assert_eq!(
        pagination.total_count, 2,
        "total_count が実際のユーザー数 2 と一致しなければならない"
    );
    assert!(
        !pagination.has_next,
        "全件が 1 ページに収まるため has_next が false でなければならない"
    );
}

// ページネーション指定で list_users gRPC を呼び出すとページ情報付きで返ることを検証する
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
    // gRPC レスポンス取得: ページネーション付きのユーザー一覧取得に失敗してはならない
    let resp = svc
        .list_users(req)
        .await
        .expect("ページネーション指定での list_users gRPC 呼び出しが失敗しました");

    assert_eq!(
        resp.users.len(),
        2,
        "page_size=10 の場合、テストデータ 2 件が全て返らなければならない"
    );
    // ページネーションメタデータの検証
    let pagination = resp
        .pagination
        .expect("ページネーション付きレスポンスに pagination フィールドが存在しなければならない");
    assert_eq!(
        pagination.page, 1,
        "レスポンスの page が要求値 1 と一致しなければならない"
    );
    assert_eq!(
        pagination.page_size, 10,
        "レスポンスの page_size が要求値 10 と一致しなければならない"
    );
    assert_eq!(
        pagination.total_count, 2,
        "total_count がリポジトリの総ユーザー数 2 と一致しなければならない"
    );
}

// 権限を持つロールでの check_permission gRPC 呼び出しが許可されることを検証する
#[tokio::test]
async fn test_check_permission_grpc_success() {
    let svc = make_grpc_service(true);

    let req = CheckPermissionRequest {
        user_id: Some("grpc-user-1".to_string()),
        permission: "read".to_string(),
        resource: "audit_logs".to_string(),
        roles: vec!["sys_auditor".to_string()],
    };
    // gRPC レスポンス取得: 権限チェック gRPC 呼び出しに失敗してはならない
    let resp = svc
        .check_permission(req)
        .await
        .expect("check_permission gRPC 呼び出しが失敗しました");

    assert!(
        resp.allowed,
        "sys_auditor ロールは audit_logs への read 権限を持つため allowed が true でなければならない"
    );
    assert!(
        resp.reason.is_empty(),
        "許可された場合は reason が空でなければならない"
    );
}

// 権限を持たないロールでの check_permission gRPC 呼び出しが拒否されることを検証する
#[tokio::test]
async fn test_check_permission_grpc_denied() {
    let svc = make_grpc_service(true);

    let req = CheckPermissionRequest {
        user_id: Some("grpc-user-1".to_string()),
        permission: "admin".to_string(),
        resource: "users".to_string(),
        roles: vec!["user".to_string()],
    };
    // gRPC レスポンス取得: 権限チェック gRPC 呼び出しに失敗してはならない
    let resp = svc
        .check_permission(req)
        .await
        .expect("check_permission gRPC 呼び出しが失敗しました");

    assert!(
        !resp.allowed,
        "一般ユーザーロールは users への admin 権限を持たないため allowed が false でなければならない"
    );
    assert!(
        resp.reason.contains("insufficient permissions"),
        "権限不足の場合は reason に 'insufficient permissions' が含まれなければならない: 実際={}",
        resp.reason
    );
}

// 存在するユーザーの get_user_roles gRPC 呼び出しでロール一覧が返ることを検証する
#[tokio::test]
async fn test_get_user_roles_grpc_success() {
    let svc = make_grpc_service(true);

    let req = GetUserRolesRequest {
        user_id: "grpc-user-1".to_string(),
    };
    // gRPC レスポンス取得: ユーザーロール取得 gRPC 呼び出しに失敗してはならない
    let resp = svc
        .get_user_roles(req)
        .await
        .expect("get_user_roles gRPC 呼び出しが失敗しました");

    assert_eq!(
        resp.user_id, "grpc-user-1",
        "レスポンスの user_id が要求した 'grpc-user-1' と一致しなければならない"
    );
    assert_eq!(
        resp.realm_roles.len(),
        2,
        "grpc-user-1 は realm ロールを 2 件持つため長さが 2 でなければならない"
    );
    assert_eq!(
        resp.realm_roles[0].name, "user",
        "1 件目の realm ロール名が 'user' でなければならない"
    );
    assert_eq!(
        resp.realm_roles[1].name, "sys_auditor",
        "2 件目の realm ロール名が 'sys_auditor' でなければならない"
    );
    assert_eq!(
        resp.client_roles.len(),
        1,
        "grpc-user-1 はクライアントロールを 1 件持つため長さが 1 でなければならない"
    );
    assert!(
        resp.client_roles.contains_key("task-server"),
        "クライアントロールに 'task-server' エントリが存在しなければならない"
    );
    assert_eq!(
        resp.client_roles["task-server"].roles.len(),
        1,
        "'task-server' クライアントのロール数が 1 でなければならない"
    );
    assert_eq!(
        resp.client_roles["task-server"].roles[0].name, "read",
        "'task-server' クライアントのロール名が 'read' でなければならない"
    );
}
