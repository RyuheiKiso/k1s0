use std::sync::Arc;

use async_trait::async_trait;

use k1s0_navigation_server::domain::entity::navigation::{
    Guard, GuardType, NavigationConfig, Param, ParamType, Route, TransitionType,
};
use k1s0_navigation_server::infrastructure::navigation_loader::NavigationConfigLoader;
use k1s0_navigation_server::usecase::get_navigation::{
    GetNavigationUseCase, NavigationError, NavigationTokenVerifier,
};

// ============================================================
// Stub implementations
// ============================================================

/// In-memory stub for NavigationConfigLoader.
struct StubNavigationConfigLoader {
    config: Option<NavigationConfig>,
    should_error: bool,
}

impl StubNavigationConfigLoader {
    fn new(config: NavigationConfig) -> Self {
        Self {
            config: Some(config),
            should_error: false,
        }
    }

    fn with_error() -> Self {
        Self {
            config: None,
            should_error: true,
        }
    }
}

impl NavigationConfigLoader for StubNavigationConfigLoader {
    fn load(&self) -> anyhow::Result<NavigationConfig> {
        if self.should_error {
            return Err(anyhow::anyhow!("config file not found"));
        }
        Ok(self.config.clone().unwrap())
    }
}

/// In-memory stub for NavigationTokenVerifier.
struct StubTokenVerifier {
    roles: Vec<String>,
    should_error: bool,
}

impl StubTokenVerifier {
    fn with_roles(roles: Vec<String>) -> Self {
        Self {
            roles,
            should_error: false,
        }
    }

    fn with_error() -> Self {
        Self {
            roles: vec![],
            should_error: true,
        }
    }
}

#[async_trait]
impl NavigationTokenVerifier for StubTokenVerifier {
    async fn verify_roles(&self, _bearer_token: &str) -> anyhow::Result<Vec<String>> {
        if self.should_error {
            return Err(anyhow::anyhow!(
                "token verification failed: invalid signature"
            ));
        }
        Ok(self.roles.clone())
    }
}

// ============================================================
// Helper: build a rich NavigationConfig for testing
// ============================================================

fn make_route(
    id: &str,
    path: &str,
    component_id: Option<&str>,
    guards: Vec<&str>,
    children: Vec<Route>,
) -> Route {
    Route {
        id: id.to_string(),
        path: path.to_string(),
        component_id: component_id.map(|s| s.to_string()),
        guards: guards.into_iter().map(|s| s.to_string()).collect(),
        transition: None,
        transition_duration_ms: 300,
        redirect_to: None,
        children,
        params: vec![],
    }
}

fn make_full_config() -> NavigationConfig {
    NavigationConfig {
        version: 1,
        guards: vec![
            Guard {
                id: "auth_required".to_string(),
                guard_type: GuardType::AuthRequired,
                redirect_to: "/login".to_string(),
                roles: vec![],
            },
            Guard {
                id: "admin_only".to_string(),
                guard_type: GuardType::RoleRequired,
                roles: vec!["admin".to_string()],
                redirect_to: "/forbidden".to_string(),
            },
            Guard {
                id: "editor_only".to_string(),
                guard_type: GuardType::RoleRequired,
                roles: vec!["editor".to_string()],
                redirect_to: "/forbidden".to_string(),
            },
            Guard {
                id: "guest_only".to_string(),
                guard_type: GuardType::RedirectIfAuthenticated,
                redirect_to: "/dashboard".to_string(),
                roles: vec![],
            },
        ],
        routes: vec![
            // Public root redirect
            make_route("root", "/", None, vec![], vec![]),
            // Guest-only login page
            make_route(
                "login",
                "/login",
                Some("LoginPage"),
                vec!["guest_only"],
                vec![],
            ),
            // Auth-required dashboard
            make_route(
                "dashboard",
                "/dashboard",
                Some("DashboardPage"),
                vec!["auth_required"],
                vec![],
            ),
            // Admin section with children
            make_route(
                "admin",
                "/admin",
                Some("AdminPage"),
                vec!["auth_required", "admin_only"],
                vec![
                    make_route(
                        "admin_users",
                        "/admin/users",
                        Some("AdminUsersPage"),
                        vec!["auth_required", "admin_only"],
                        vec![],
                    ),
                    make_route(
                        "admin_settings",
                        "/admin/settings",
                        Some("AdminSettingsPage"),
                        vec!["auth_required", "admin_only"],
                        vec![],
                    ),
                ],
            ),
            // Editor section
            make_route(
                "editor",
                "/editor",
                Some("EditorPage"),
                vec!["auth_required", "editor_only"],
                vec![],
            ),
            // Public about page
            make_route("about", "/about", Some("AboutPage"), vec![], vec![]),
        ],
    }
}

// ============================================================
// GetNavigation usecase tests
// ============================================================

#[tokio::test]
async fn get_navigation_empty_token_returns_public_and_guest_routes() {
    let loader = StubNavigationConfigLoader::new(make_full_config());
    let uc = GetNavigationUseCase::new(Arc::new(loader), None);

    let result = uc.execute("").await.unwrap();
    let ids: Vec<&str> = result.routes.iter().map(|r| r.id.as_str()).collect();

    assert!(ids.contains(&"root"), "root is public");
    assert!(
        ids.contains(&"login"),
        "login is guest_only, visible to unauthenticated"
    );
    assert!(ids.contains(&"about"), "about is public");
    assert!(!ids.contains(&"dashboard"), "dashboard needs auth");
    assert!(!ids.contains(&"admin"), "admin needs auth+admin role");
    assert!(!ids.contains(&"editor"), "editor needs auth+editor role");
}

#[tokio::test]
async fn get_navigation_authenticated_user_without_roles() {
    let loader = StubNavigationConfigLoader::new(make_full_config());
    let verifier = StubTokenVerifier::with_roles(vec!["user".to_string()]);
    let uc = GetNavigationUseCase::new(Arc::new(loader), Some(Arc::new(verifier)));

    let result = uc.execute("valid-token").await.unwrap();
    let ids: Vec<&str> = result.routes.iter().map(|r| r.id.as_str()).collect();

    assert!(ids.contains(&"root"), "root is always accessible");
    assert!(ids.contains(&"dashboard"), "dashboard is auth-required");
    assert!(ids.contains(&"about"), "about is public");
    assert!(
        !ids.contains(&"login"),
        "login blocked for authenticated users"
    );
    assert!(!ids.contains(&"admin"), "admin requires admin role");
    assert!(!ids.contains(&"editor"), "editor requires editor role");
}

#[tokio::test]
async fn get_navigation_admin_user_sees_admin_routes_with_children() {
    let loader = StubNavigationConfigLoader::new(make_full_config());
    let verifier = StubTokenVerifier::with_roles(vec!["admin".to_string()]);
    let uc = GetNavigationUseCase::new(Arc::new(loader), Some(Arc::new(verifier)));

    let result = uc.execute("admin-token").await.unwrap();
    let ids: Vec<&str> = result.routes.iter().map(|r| r.id.as_str()).collect();

    assert!(
        ids.contains(&"admin"),
        "admin user should see admin section"
    );
    assert!(
        ids.contains(&"dashboard"),
        "admin user should see dashboard"
    );

    let admin_route = result.routes.iter().find(|r| r.id == "admin").unwrap();
    assert_eq!(admin_route.children.len(), 2, "admin has 2 child routes");
    let child_ids: Vec<&str> = admin_route.children.iter().map(|r| r.id.as_str()).collect();
    assert!(child_ids.contains(&"admin_users"));
    assert!(child_ids.contains(&"admin_settings"));
}

#[tokio::test]
async fn get_navigation_editor_user_sees_editor_but_not_admin() {
    let loader = StubNavigationConfigLoader::new(make_full_config());
    let verifier = StubTokenVerifier::with_roles(vec!["editor".to_string()]);
    let uc = GetNavigationUseCase::new(Arc::new(loader), Some(Arc::new(verifier)));

    let result = uc.execute("editor-token").await.unwrap();
    let ids: Vec<&str> = result.routes.iter().map(|r| r.id.as_str()).collect();

    assert!(
        ids.contains(&"editor"),
        "editor user should see editor section"
    );
    assert!(
        ids.contains(&"dashboard"),
        "editor user should see dashboard"
    );
    assert!(
        !ids.contains(&"admin"),
        "editor user should NOT see admin section"
    );
}

#[tokio::test]
async fn get_navigation_multi_role_user_sees_all_permitted() {
    let loader = StubNavigationConfigLoader::new(make_full_config());
    let verifier = StubTokenVerifier::with_roles(vec!["admin".to_string(), "editor".to_string()]);
    let uc = GetNavigationUseCase::new(Arc::new(loader), Some(Arc::new(verifier)));

    let result = uc.execute("super-token").await.unwrap();
    let ids: Vec<&str> = result.routes.iter().map(|r| r.id.as_str()).collect();

    assert!(ids.contains(&"admin"));
    assert!(ids.contains(&"editor"));
    assert!(ids.contains(&"dashboard"));
}

#[tokio::test]
async fn get_navigation_config_load_error() {
    let loader = StubNavigationConfigLoader::with_error();
    let uc = GetNavigationUseCase::new(Arc::new(loader), None);

    let result = uc.execute("").await;
    assert!(result.is_err());
    match result.unwrap_err() {
        NavigationError::ConfigLoad(msg) => {
            assert!(msg.contains("config file not found"));
        }
        other => panic!("expected ConfigLoad error, got: {:?}", other),
    }
}

#[tokio::test]
async fn get_navigation_token_verification_error() {
    let loader = StubNavigationConfigLoader::new(make_full_config());
    let verifier = StubTokenVerifier::with_error();
    let uc = GetNavigationUseCase::new(Arc::new(loader), Some(Arc::new(verifier)));

    let result = uc.execute("bad-token").await;
    assert!(result.is_err());
    match result.unwrap_err() {
        NavigationError::TokenVerification(msg) => {
            assert!(msg.contains("invalid signature"));
        }
        other => panic!("expected TokenVerification error, got: {:?}", other),
    }
}

#[tokio::test]
async fn get_navigation_token_without_verifier_treats_as_unauthenticated() {
    let loader = StubNavigationConfigLoader::new(make_full_config());
    let uc = GetNavigationUseCase::new(Arc::new(loader), None);

    // Even with a token, if no verifier is configured, user is treated as unauthenticated
    let result = uc.execute("some-token").await.unwrap();
    let ids: Vec<&str> = result.routes.iter().map(|r| r.id.as_str()).collect();

    assert!(
        ids.contains(&"login"),
        "guest routes visible when no verifier"
    );
    assert!(
        !ids.contains(&"dashboard"),
        "auth routes hidden when no verifier"
    );
}

#[tokio::test]
async fn get_navigation_only_used_guards_returned() {
    let loader = StubNavigationConfigLoader::new(make_full_config());
    let uc = GetNavigationUseCase::new(Arc::new(loader), None);

    let result = uc.execute("").await.unwrap();
    let guard_ids: Vec<&str> = result.guards.iter().map(|g| g.id.as_str()).collect();

    // Unauthenticated: only guest_only guard is used by the login route
    assert!(guard_ids.contains(&"guest_only"));
    assert!(!guard_ids.contains(&"auth_required"));
    assert!(!guard_ids.contains(&"admin_only"));
    assert!(!guard_ids.contains(&"editor_only"));
}

#[tokio::test]
async fn get_navigation_empty_config() {
    let config = NavigationConfig {
        version: 1,
        guards: vec![],
        routes: vec![],
    };
    let loader = StubNavigationConfigLoader::new(config);
    let uc = GetNavigationUseCase::new(Arc::new(loader), None);

    let result = uc.execute("").await.unwrap();
    assert!(result.routes.is_empty());
    assert!(result.guards.is_empty());
}

#[tokio::test]
async fn get_navigation_children_filtered_independently_from_parent() {
    // A public parent with mixed children: some auth-required, some public
    let config = NavigationConfig {
        version: 1,
        guards: vec![Guard {
            id: "auth_required".to_string(),
            guard_type: GuardType::AuthRequired,
            redirect_to: "/login".to_string(),
            roles: vec![],
        }],
        routes: vec![make_route(
            "parent",
            "/parent",
            Some("ParentPage"),
            vec![],
            vec![
                make_route(
                    "public_child",
                    "/parent/public",
                    Some("PublicChild"),
                    vec![],
                    vec![],
                ),
                make_route(
                    "auth_child",
                    "/parent/auth",
                    Some("AuthChild"),
                    vec!["auth_required"],
                    vec![],
                ),
            ],
        )],
    };
    let loader = StubNavigationConfigLoader::new(config);
    let uc = GetNavigationUseCase::new(Arc::new(loader), None);

    let result = uc.execute("").await.unwrap();
    assert_eq!(result.routes.len(), 1);
    let parent = &result.routes[0];
    assert_eq!(parent.children.len(), 1, "only public child should remain");
    assert_eq!(parent.children[0].id, "public_child");
}

#[tokio::test]
async fn get_navigation_deeply_nested_routes() {
    let config = NavigationConfig {
        version: 1,
        guards: vec![Guard {
            id: "auth_required".to_string(),
            guard_type: GuardType::AuthRequired,
            redirect_to: "/login".to_string(),
            roles: vec![],
        }],
        routes: vec![make_route(
            "l1",
            "/l1",
            Some("L1"),
            vec![],
            vec![make_route(
                "l2",
                "/l1/l2",
                Some("L2"),
                vec![],
                vec![make_route(
                    "l3_public",
                    "/l1/l2/l3",
                    Some("L3"),
                    vec![],
                    vec![],
                )],
            )],
        )],
    };
    let loader = StubNavigationConfigLoader::new(config);
    let uc = GetNavigationUseCase::new(Arc::new(loader), None);

    let result = uc.execute("").await.unwrap();
    assert_eq!(result.routes.len(), 1);
    assert_eq!(result.routes[0].children.len(), 1);
    assert_eq!(result.routes[0].children[0].children.len(), 1);
    assert_eq!(result.routes[0].children[0].children[0].id, "l3_public");
}

#[tokio::test]
async fn get_navigation_route_with_params_and_transition() {
    let config = NavigationConfig {
        version: 1,
        guards: vec![],
        routes: vec![Route {
            id: "user_detail".to_string(),
            path: "/users/:id".to_string(),
            component_id: Some("UserDetailPage".to_string()),
            guards: vec![],
            transition: Some(TransitionType::Slide),
            transition_duration_ms: 500,
            redirect_to: None,
            children: vec![],
            params: vec![Param {
                name: "id".to_string(),
                param_type: ParamType::Uuid,
            }],
        }],
    };
    let loader = StubNavigationConfigLoader::new(config);
    let uc = GetNavigationUseCase::new(Arc::new(loader), None);

    let result = uc.execute("").await.unwrap();
    assert_eq!(result.routes.len(), 1);
    let route = &result.routes[0];
    assert_eq!(route.params.len(), 1);
    assert_eq!(route.params[0].param_type, ParamType::Uuid);
    assert_eq!(route.transition, Some(TransitionType::Slide));
    assert_eq!(route.transition_duration_ms, 500);
}

#[tokio::test]
async fn get_navigation_redirect_if_authenticated_guard_allows_unauthenticated() {
    // Specifically test RedirectIfAuthenticated semantics
    let config = NavigationConfig {
        version: 1,
        guards: vec![Guard {
            id: "guest_only".to_string(),
            guard_type: GuardType::RedirectIfAuthenticated,
            redirect_to: "/dashboard".to_string(),
            roles: vec![],
        }],
        routes: vec![
            make_route(
                "register",
                "/register",
                Some("RegisterPage"),
                vec!["guest_only"],
                vec![],
            ),
            make_route("public", "/public", Some("PublicPage"), vec![], vec![]),
        ],
    };
    let loader = StubNavigationConfigLoader::new(config);

    // Unauthenticated: both routes visible
    let uc = GetNavigationUseCase::new(Arc::new(loader), None);
    let result = uc.execute("").await.unwrap();
    let ids: Vec<&str> = result.routes.iter().map(|r| r.id.as_str()).collect();
    assert!(ids.contains(&"register"));
    assert!(ids.contains(&"public"));
}

#[tokio::test]
async fn get_navigation_redirect_if_authenticated_blocks_authenticated() {
    let config = NavigationConfig {
        version: 1,
        guards: vec![Guard {
            id: "guest_only".to_string(),
            guard_type: GuardType::RedirectIfAuthenticated,
            redirect_to: "/dashboard".to_string(),
            roles: vec![],
        }],
        routes: vec![make_route(
            "register",
            "/register",
            Some("RegisterPage"),
            vec!["guest_only"],
            vec![],
        )],
    };
    let loader = StubNavigationConfigLoader::new(config);
    let verifier = StubTokenVerifier::with_roles(vec![]);
    let uc = GetNavigationUseCase::new(Arc::new(loader), Some(Arc::new(verifier)));

    let result = uc.execute("valid-token").await.unwrap();
    assert!(
        result.routes.is_empty(),
        "register page should be hidden for authenticated users"
    );
}

#[tokio::test]
async fn get_navigation_unknown_guard_id_allows_access() {
    // Route references a guard that does not exist in guards list -> allowed
    let config = NavigationConfig {
        version: 1,
        guards: vec![],
        routes: vec![make_route(
            "mystery",
            "/mystery",
            Some("MysteryPage"),
            vec!["nonexistent_guard"],
            vec![],
        )],
    };
    let loader = StubNavigationConfigLoader::new(config);
    let uc = GetNavigationUseCase::new(Arc::new(loader), None);

    let result = uc.execute("").await.unwrap();
    assert_eq!(
        result.routes.len(),
        1,
        "unknown guard should not block the route"
    );
}

#[tokio::test]
async fn get_navigation_admin_guards_returned_for_admin_user() {
    let loader = StubNavigationConfigLoader::new(make_full_config());
    let verifier = StubTokenVerifier::with_roles(vec!["admin".to_string()]);
    let uc = GetNavigationUseCase::new(Arc::new(loader), Some(Arc::new(verifier)));

    let result = uc.execute("admin-token").await.unwrap();
    let guard_ids: Vec<&str> = result.guards.iter().map(|g| g.id.as_str()).collect();

    assert!(
        guard_ids.contains(&"auth_required"),
        "admin routes use auth_required"
    );
    assert!(
        guard_ids.contains(&"admin_only"),
        "admin routes use admin_only"
    );
}

// ============================================================
// 追加テスト: ガードの複合条件テスト
// ============================================================

/// auth_required + admin_only の両ガードがある場合、admin ロールなしではブロックされる。
#[tokio::test]
async fn get_navigation_combined_guards_auth_without_admin_blocks() {
    let loader = StubNavigationConfigLoader::new(make_full_config());
    let verifier = StubTokenVerifier::with_roles(vec!["user".to_string()]);
    let uc = GetNavigationUseCase::new(Arc::new(loader), Some(Arc::new(verifier)));

    let result = uc.execute("user-token").await.unwrap();
    let ids: Vec<&str> = result.routes.iter().map(|r| r.id.as_str()).collect();

    // user ロールのみでは admin セクションにアクセスできない
    assert!(!ids.contains(&"admin"));
    assert!(!ids.contains(&"editor"));
    // ただし dashboard にはアクセスできる
    assert!(ids.contains(&"dashboard"));
}

/// 3つのロールを持つユーザーが全ルートにアクセスできることを確認する。
#[tokio::test]
async fn get_navigation_user_with_all_roles_sees_everything() {
    let loader = StubNavigationConfigLoader::new(make_full_config());
    let verifier = StubTokenVerifier::with_roles(vec![
        "user".to_string(),
        "admin".to_string(),
        "editor".to_string(),
    ]);
    let uc = GetNavigationUseCase::new(Arc::new(loader), Some(Arc::new(verifier)));

    let result = uc.execute("super-user-token").await.unwrap();
    let ids: Vec<&str> = result.routes.iter().map(|r| r.id.as_str()).collect();

    assert!(ids.contains(&"root"));
    assert!(ids.contains(&"dashboard"));
    assert!(ids.contains(&"admin"));
    assert!(ids.contains(&"editor"));
    assert!(ids.contains(&"about"));
    // ログインページは認証済みユーザーには表示されない
    assert!(!ids.contains(&"login"));
}

// ============================================================
// ルートの可視性テスト
// ============================================================

/// 全ルートが public（ガードなし）の場合、全ルートが見える。
#[tokio::test]
async fn get_navigation_all_public_routes_visible_to_anyone() {
    let config = NavigationConfig {
        version: 1,
        guards: vec![],
        routes: vec![
            make_route("home", "/", Some("HomePage"), vec![], vec![]),
            make_route("about", "/about", Some("AboutPage"), vec![], vec![]),
            make_route("contact", "/contact", Some("ContactPage"), vec![], vec![]),
        ],
    };
    let loader = StubNavigationConfigLoader::new(config);
    let uc = GetNavigationUseCase::new(Arc::new(loader), None);

    let result = uc.execute("").await.unwrap();
    assert_eq!(result.routes.len(), 3);
    assert!(result.guards.is_empty());
}

/// redirect_to が設定されたルートが正しく返される。
#[tokio::test]
async fn get_navigation_route_with_redirect_to() {
    let config = NavigationConfig {
        version: 1,
        guards: vec![],
        routes: vec![Route {
            id: "old_page".to_string(),
            path: "/old".to_string(),
            component_id: None,
            guards: vec![],
            transition: None,
            transition_duration_ms: 300,
            redirect_to: Some("/new".to_string()),
            children: vec![],
            params: vec![],
        }],
    };
    let loader = StubNavigationConfigLoader::new(config);
    let uc = GetNavigationUseCase::new(Arc::new(loader), None);

    let result = uc.execute("").await.unwrap();
    assert_eq!(result.routes.len(), 1);
    assert_eq!(result.routes[0].redirect_to, Some("/new".to_string()));
}

// ============================================================
// 深いネスティングテスト
// ============================================================

/// 5階層のネストされたルートが正しくフィルタリングされる。
#[tokio::test]
async fn get_navigation_five_level_nesting() {
    let config = NavigationConfig {
        version: 1,
        guards: vec![],
        routes: vec![make_route(
            "l1",
            "/l1",
            Some("L1"),
            vec![],
            vec![make_route(
                "l2",
                "/l1/l2",
                Some("L2"),
                vec![],
                vec![make_route(
                    "l3",
                    "/l1/l2/l3",
                    Some("L3"),
                    vec![],
                    vec![make_route(
                        "l4",
                        "/l1/l2/l3/l4",
                        Some("L4"),
                        vec![],
                        vec![make_route(
                            "l5",
                            "/l1/l2/l3/l4/l5",
                            Some("L5"),
                            vec![],
                            vec![],
                        )],
                    )],
                )],
            )],
        )],
    };
    let loader = StubNavigationConfigLoader::new(config);
    let uc = GetNavigationUseCase::new(Arc::new(loader), None);

    let result = uc.execute("").await.unwrap();
    assert_eq!(result.routes.len(), 1);

    // 全5階層をたどる
    let l1 = &result.routes[0];
    assert_eq!(l1.id, "l1");
    assert_eq!(l1.children.len(), 1);

    let l2 = &l1.children[0];
    assert_eq!(l2.id, "l2");
    assert_eq!(l2.children.len(), 1);

    let l3 = &l2.children[0];
    assert_eq!(l3.id, "l3");
    assert_eq!(l3.children.len(), 1);

    let l4 = &l3.children[0];
    assert_eq!(l4.id, "l4");
    assert_eq!(l4.children.len(), 1);

    let l5 = &l4.children[0];
    assert_eq!(l5.id, "l5");
    assert!(l5.children.is_empty());
}

/// 深いネストで中間ルートが auth_required の場合、未認証ユーザーからブロックされる。
#[tokio::test]
async fn get_navigation_deep_nesting_with_auth_at_middle_level() {
    let config = NavigationConfig {
        version: 1,
        guards: vec![Guard {
            id: "auth_required".to_string(),
            guard_type: GuardType::AuthRequired,
            redirect_to: "/login".to_string(),
            roles: vec![],
        }],
        routes: vec![make_route(
            "l1",
            "/l1",
            Some("L1"),
            vec![],
            vec![
                make_route("l2_public", "/l1/public", Some("L2Public"), vec![], vec![]),
                make_route(
                    "l2_auth",
                    "/l1/auth",
                    Some("L2Auth"),
                    vec!["auth_required"],
                    vec![make_route(
                        "l3_under_auth",
                        "/l1/auth/child",
                        Some("L3"),
                        vec!["auth_required"],
                        vec![],
                    )],
                ),
            ],
        )],
    };
    let loader = StubNavigationConfigLoader::new(config);
    let uc = GetNavigationUseCase::new(Arc::new(loader), None);

    let result = uc.execute("").await.unwrap();
    assert_eq!(result.routes.len(), 1);
    let l1 = &result.routes[0];
    // public child のみ残る
    assert_eq!(l1.children.len(), 1);
    assert_eq!(l1.children[0].id, "l2_public");
}

// ============================================================
// ルートパラメータテスト
// ============================================================

/// 複数パラメータを持つルートが正しく返される。
#[tokio::test]
async fn get_navigation_route_with_multiple_params() {
    let config = NavigationConfig {
        version: 1,
        guards: vec![],
        routes: vec![Route {
            id: "user_post".to_string(),
            path: "/users/:user_id/posts/:post_id".to_string(),
            component_id: Some("UserPostPage".to_string()),
            guards: vec![],
            transition: Some(TransitionType::Fade),
            transition_duration_ms: 200,
            redirect_to: None,
            children: vec![],
            params: vec![
                Param {
                    name: "user_id".to_string(),
                    param_type: ParamType::Uuid,
                },
                Param {
                    name: "post_id".to_string(),
                    param_type: ParamType::Int,
                },
            ],
        }],
    };
    let loader = StubNavigationConfigLoader::new(config);
    let uc = GetNavigationUseCase::new(Arc::new(loader), None);

    let result = uc.execute("").await.unwrap();
    assert_eq!(result.routes.len(), 1);
    let route = &result.routes[0];
    assert_eq!(route.params.len(), 2);
    assert_eq!(route.params[0].name, "user_id");
    assert_eq!(route.params[0].param_type, ParamType::Uuid);
    assert_eq!(route.params[1].name, "post_id");
    assert_eq!(route.params[1].param_type, ParamType::Int);
}

/// String 型のパラメータを持つルートが正しく返される。
#[tokio::test]
async fn get_navigation_route_with_string_param() {
    let config = NavigationConfig {
        version: 1,
        guards: vec![],
        routes: vec![Route {
            id: "search".to_string(),
            path: "/search/:query".to_string(),
            component_id: Some("SearchPage".to_string()),
            guards: vec![],
            transition: None,
            transition_duration_ms: 300,
            redirect_to: None,
            children: vec![],
            params: vec![Param {
                name: "query".to_string(),
                param_type: ParamType::String,
            }],
        }],
    };
    let loader = StubNavigationConfigLoader::new(config);
    let uc = GetNavigationUseCase::new(Arc::new(loader), None);

    let result = uc.execute("").await.unwrap();
    assert_eq!(result.routes[0].params[0].param_type, ParamType::String);
}

// ============================================================
// トランジションテスト
// ============================================================

/// Modal トランジションを持つルートが正しく返される。
#[tokio::test]
async fn get_navigation_modal_transition() {
    let config = NavigationConfig {
        version: 1,
        guards: vec![],
        routes: vec![Route {
            id: "modal_dialog".to_string(),
            path: "/dialog".to_string(),
            component_id: Some("DialogPage".to_string()),
            guards: vec![],
            transition: Some(TransitionType::Modal),
            transition_duration_ms: 150,
            redirect_to: None,
            children: vec![],
            params: vec![],
        }],
    };
    let loader = StubNavigationConfigLoader::new(config);
    let uc = GetNavigationUseCase::new(Arc::new(loader), None);

    let result = uc.execute("").await.unwrap();
    assert_eq!(result.routes[0].transition, Some(TransitionType::Modal));
    assert_eq!(result.routes[0].transition_duration_ms, 150);
}

/// デフォルトトランジション（None）のルートが正しく返される。
#[tokio::test]
async fn get_navigation_no_transition_uses_default_duration() {
    let config = NavigationConfig {
        version: 1,
        guards: vec![],
        routes: vec![make_route(
            "simple",
            "/simple",
            Some("SimplePage"),
            vec![],
            vec![],
        )],
    };
    let loader = StubNavigationConfigLoader::new(config);
    let uc = GetNavigationUseCase::new(Arc::new(loader), None);

    let result = uc.execute("").await.unwrap();
    assert!(result.routes[0].transition.is_none());
    assert_eq!(result.routes[0].transition_duration_ms, 300);
}

// ============================================================
// 複数ガード参照テスト
// ============================================================

/// 複数の unknown ガードが参照されたルートは許可される。
#[tokio::test]
async fn get_navigation_multiple_unknown_guards_allowed() {
    let config = NavigationConfig {
        version: 1,
        guards: vec![],
        routes: vec![make_route(
            "mystery",
            "/mystery",
            Some("MysteryPage"),
            vec!["unknown_guard_1", "unknown_guard_2"],
            vec![],
        )],
    };
    let loader = StubNavigationConfigLoader::new(config);
    let uc = GetNavigationUseCase::new(Arc::new(loader), None);

    let result = uc.execute("").await.unwrap();
    assert_eq!(result.routes.len(), 1);
}

/// 認証済みユーザーのガードリストに editor_only が含まれることを確認する。
#[tokio::test]
async fn get_navigation_editor_guards_returned_for_editor_user() {
    let loader = StubNavigationConfigLoader::new(make_full_config());
    let verifier = StubTokenVerifier::with_roles(vec!["editor".to_string()]);
    let uc = GetNavigationUseCase::new(Arc::new(loader), Some(Arc::new(verifier)));

    let result = uc.execute("editor-token").await.unwrap();
    let guard_ids: Vec<&str> = result.guards.iter().map(|g| g.id.as_str()).collect();

    assert!(guard_ids.contains(&"auth_required"));
    assert!(guard_ids.contains(&"editor_only"));
    assert!(!guard_ids.contains(&"admin_only"));
}

// ============================================================
// 大規模ルートツリーのテスト
// ============================================================

/// 多数のルートを持つ設定でもフィルタリングが正しく動作する。
#[tokio::test]
async fn get_navigation_large_route_tree() {
    let mut routes = Vec::new();
    for i in 0..20 {
        routes.push(make_route(
            &format!("route_{}", i),
            &format!("/page/{}", i),
            Some(&format!("Page{}", i)),
            vec![],
            vec![],
        ));
    }
    let config = NavigationConfig {
        version: 1,
        guards: vec![],
        routes,
    };
    let loader = StubNavigationConfigLoader::new(config);
    let uc = GetNavigationUseCase::new(Arc::new(loader), None);

    let result = uc.execute("").await.unwrap();
    assert_eq!(result.routes.len(), 20);
}

// ============================================================
// 子ルートの認証依存フィルタリングテスト
// ============================================================

/// 親ルートが public で子ルートの一部に role_required がある場合のフィルタリング。
#[tokio::test]
async fn get_navigation_public_parent_with_role_required_children() {
    let config = NavigationConfig {
        version: 1,
        guards: vec![
            Guard {
                id: "auth_required".to_string(),
                guard_type: GuardType::AuthRequired,
                redirect_to: "/login".to_string(),
                roles: vec![],
            },
            Guard {
                id: "admin_only".to_string(),
                guard_type: GuardType::RoleRequired,
                roles: vec!["admin".to_string()],
                redirect_to: "/forbidden".to_string(),
            },
        ],
        routes: vec![make_route(
            "settings",
            "/settings",
            Some("SettingsPage"),
            vec![],
            vec![
                make_route(
                    "profile",
                    "/settings/profile",
                    Some("ProfilePage"),
                    vec![],
                    vec![],
                ),
                make_route(
                    "admin_settings",
                    "/settings/admin",
                    Some("AdminSettingsPage"),
                    vec!["auth_required", "admin_only"],
                    vec![],
                ),
            ],
        )],
    };
    let loader = StubNavigationConfigLoader::new(config);
    // 認証済みだが admin ロールなし
    let verifier = StubTokenVerifier::with_roles(vec!["user".to_string()]);
    let uc = GetNavigationUseCase::new(Arc::new(loader), Some(Arc::new(verifier)));

    let result = uc.execute("user-token").await.unwrap();
    assert_eq!(result.routes.len(), 1);
    let settings = &result.routes[0];
    // profile は public なので見える、admin_settings は admin 専用なので見えない
    assert_eq!(settings.children.len(), 1);
    assert_eq!(settings.children[0].id, "profile");
}

/// 全ルートが auth_required で未認証ユーザーの場合、空の結果が返される。
#[tokio::test]
async fn get_navigation_all_auth_required_unauthenticated_returns_empty() {
    let config = NavigationConfig {
        version: 1,
        guards: vec![Guard {
            id: "auth_required".to_string(),
            guard_type: GuardType::AuthRequired,
            redirect_to: "/login".to_string(),
            roles: vec![],
        }],
        routes: vec![
            make_route(
                "dashboard",
                "/dashboard",
                Some("Dashboard"),
                vec!["auth_required"],
                vec![],
            ),
            make_route(
                "profile",
                "/profile",
                Some("Profile"),
                vec!["auth_required"],
                vec![],
            ),
        ],
    };
    let loader = StubNavigationConfigLoader::new(config);
    let uc = GetNavigationUseCase::new(Arc::new(loader), None);

    let result = uc.execute("").await.unwrap();
    assert!(result.routes.is_empty());
    assert!(result.guards.is_empty());
}
