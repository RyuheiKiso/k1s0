use std::collections::HashSet;

use crate::domain::entity::navigation::{Guard, GuardType, NavigationConfig, Route};

/// ユーザーコンテキスト。
#[derive(Debug, Clone)]
pub struct UserContext {
    pub authenticated: bool,
    pub roles: Vec<String>,
}

/// フィルタリング済みナビゲーション。
#[derive(Debug, Clone)]
pub struct FilteredNavigation {
    pub routes: Vec<Route>,
    pub guards: Vec<Guard>,
}

/// ナビゲーション設定をユーザーコンテキストに基づいてフィルタリングする。
pub fn filter_navigation(config: &NavigationConfig, user_ctx: &UserContext) -> FilteredNavigation {
    let filtered_routes = filter_routes(&config.routes, &config.guards, user_ctx);
    let used_guard_ids = collect_guard_ids(&filtered_routes);
    let used_guards = config
        .guards
        .iter()
        .filter(|g| used_guard_ids.contains(&g.id))
        .cloned()
        .collect();
    FilteredNavigation {
        routes: filtered_routes,
        guards: used_guards,
    }
}

fn filter_routes(routes: &[Route], guards: &[Guard], user_ctx: &UserContext) -> Vec<Route> {
    routes
        .iter()
        .filter(|route| is_route_accessible(route, guards, user_ctx))
        .map(|route| {
            let mut r = route.clone();
            r.children = filter_routes(&route.children, guards, user_ctx);
            r
        })
        .collect()
}

fn is_route_accessible(route: &Route, guards: &[Guard], user_ctx: &UserContext) -> bool {
    if route.guards.is_empty() {
        return true;
    }
    route.guards.iter().all(|guard_id| {
        if let Some(guard) = guards.iter().find(|g| g.id == *guard_id) {
            match guard.guard_type {
                GuardType::AuthRequired => user_ctx.authenticated,
                GuardType::RoleRequired => {
                    user_ctx.authenticated
                        && guard.roles.iter().any(|r| user_ctx.roles.contains(r))
                }
                GuardType::RedirectIfAuthenticated => !user_ctx.authenticated,
            }
        } else {
            true
        }
    })
}

fn collect_guard_ids(routes: &[Route]) -> HashSet<String> {
    let mut ids = HashSet::new();
    for route in routes {
        for guard_id in &route.guards {
            ids.insert(guard_id.clone());
        }
        ids.extend(collect_guard_ids(&route.children));
    }
    ids
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::navigation::{Guard, GuardType, NavigationConfig, Route};

    fn make_config() -> NavigationConfig {
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
                    id: "guest_only".to_string(),
                    guard_type: GuardType::RedirectIfAuthenticated,
                    redirect_to: "/dashboard".to_string(),
                    roles: vec![],
                },
            ],
            routes: vec![
                Route {
                    id: "root".to_string(),
                    path: "/".to_string(),
                    component_id: None,
                    guards: vec![],
                    transition: None,
                    redirect_to: Some("/dashboard".to_string()),
                    children: vec![],
                    params: vec![],
                },
                Route {
                    id: "login".to_string(),
                    path: "/login".to_string(),
                    component_id: Some("LoginPage".to_string()),
                    guards: vec!["guest_only".to_string()],
                    transition: None,
                    redirect_to: None,
                    children: vec![],
                    params: vec![],
                },
                Route {
                    id: "dashboard".to_string(),
                    path: "/dashboard".to_string(),
                    component_id: Some("DashboardPage".to_string()),
                    guards: vec!["auth_required".to_string()],
                    transition: None,
                    redirect_to: None,
                    children: vec![],
                    params: vec![],
                },
                Route {
                    id: "admin".to_string(),
                    path: "/admin".to_string(),
                    component_id: Some("AdminPage".to_string()),
                    guards: vec![
                        "auth_required".to_string(),
                        "admin_only".to_string(),
                    ],
                    transition: None,
                    redirect_to: None,
                    children: vec![Route {
                        id: "admin_users".to_string(),
                        path: "/admin/users".to_string(),
                        component_id: Some("AdminUsersPage".to_string()),
                        guards: vec![
                            "auth_required".to_string(),
                            "admin_only".to_string(),
                        ],
                        transition: None,
                        redirect_to: None,
                        children: vec![],
                        params: vec![],
                    }],
                    params: vec![],
                },
                Route {
                    id: "public".to_string(),
                    path: "/about".to_string(),
                    component_id: Some("AboutPage".to_string()),
                    guards: vec![],
                    transition: None,
                    redirect_to: None,
                    children: vec![],
                    params: vec![],
                },
            ],
        }
    }

    #[test]
    fn unauthenticated_sees_public_and_guest_routes() {
        let config = make_config();
        let ctx = UserContext {
            authenticated: false,
            roles: vec![],
        };
        let result = filter_navigation(&config, &ctx);
        let ids: Vec<&str> = result.routes.iter().map(|r| r.id.as_str()).collect();
        assert!(ids.contains(&"root"));
        assert!(ids.contains(&"login"));
        assert!(ids.contains(&"public"));
        assert!(!ids.contains(&"dashboard"));
        assert!(!ids.contains(&"admin"));
    }

    #[test]
    fn authenticated_user_sees_auth_required_routes() {
        let config = make_config();
        let ctx = UserContext {
            authenticated: true,
            roles: vec!["user".to_string()],
        };
        let result = filter_navigation(&config, &ctx);
        let ids: Vec<&str> = result.routes.iter().map(|r| r.id.as_str()).collect();
        assert!(ids.contains(&"root"));
        assert!(ids.contains(&"dashboard"));
        assert!(ids.contains(&"public"));
        assert!(!ids.contains(&"login"));
        assert!(!ids.contains(&"admin"));
    }

    #[test]
    fn admin_sees_admin_routes_with_children() {
        let config = make_config();
        let ctx = UserContext {
            authenticated: true,
            roles: vec!["admin".to_string()],
        };
        let result = filter_navigation(&config, &ctx);
        let ids: Vec<&str> = result.routes.iter().map(|r| r.id.as_str()).collect();
        assert!(ids.contains(&"admin"));
        let admin_route = result.routes.iter().find(|r| r.id == "admin").unwrap();
        assert_eq!(admin_route.children.len(), 1);
        assert_eq!(admin_route.children[0].id, "admin_users");
    }

    #[test]
    fn redirect_if_authenticated_blocks_authenticated_users() {
        let config = make_config();
        let ctx = UserContext {
            authenticated: true,
            roles: vec![],
        };
        let result = filter_navigation(&config, &ctx);
        let ids: Vec<&str> = result.routes.iter().map(|r| r.id.as_str()).collect();
        assert!(!ids.contains(&"login"));
    }

    #[test]
    fn redirect_if_authenticated_allows_unauthenticated() {
        let config = make_config();
        let ctx = UserContext {
            authenticated: false,
            roles: vec![],
        };
        let result = filter_navigation(&config, &ctx);
        let ids: Vec<&str> = result.routes.iter().map(|r| r.id.as_str()).collect();
        assert!(ids.contains(&"login"));
    }

    #[test]
    fn only_used_guards_returned() {
        let config = make_config();
        let ctx = UserContext {
            authenticated: false,
            roles: vec![],
        };
        let result = filter_navigation(&config, &ctx);
        let guard_ids: Vec<&str> = result.guards.iter().map(|g| g.id.as_str()).collect();
        assert!(guard_ids.contains(&"guest_only"));
        assert!(!guard_ids.contains(&"auth_required"));
        assert!(!guard_ids.contains(&"admin_only"));
    }

    #[test]
    fn empty_config_returns_empty() {
        let config = NavigationConfig {
            version: 1,
            guards: vec![],
            routes: vec![],
        };
        let ctx = UserContext {
            authenticated: false,
            roles: vec![],
        };
        let result = filter_navigation(&config, &ctx);
        assert!(result.routes.is_empty());
        assert!(result.guards.is_empty());
    }

    #[test]
    fn children_filtered_independently() {
        let config = NavigationConfig {
            version: 1,
            guards: vec![Guard {
                id: "auth_required".to_string(),
                guard_type: GuardType::AuthRequired,
                redirect_to: "/login".to_string(),
                roles: vec![],
            }],
            routes: vec![Route {
                id: "parent".to_string(),
                path: "/parent".to_string(),
                component_id: Some("ParentPage".to_string()),
                guards: vec![],
                transition: None,
                redirect_to: None,
                children: vec![
                    Route {
                        id: "public_child".to_string(),
                        path: "/parent/public".to_string(),
                        component_id: Some("PublicChildPage".to_string()),
                        guards: vec![],
                        transition: None,
                        redirect_to: None,
                        children: vec![],
                        params: vec![],
                    },
                    Route {
                        id: "auth_child".to_string(),
                        path: "/parent/auth".to_string(),
                        component_id: Some("AuthChildPage".to_string()),
                        guards: vec!["auth_required".to_string()],
                        transition: None,
                        redirect_to: None,
                        children: vec![],
                        params: vec![],
                    },
                ],
                params: vec![],
            }],
        };
        let ctx = UserContext {
            authenticated: false,
            roles: vec![],
        };
        let result = filter_navigation(&config, &ctx);
        assert_eq!(result.routes.len(), 1);
        let parent = &result.routes[0];
        assert_eq!(parent.children.len(), 1);
        assert_eq!(parent.children[0].id, "public_child");
    }
}
