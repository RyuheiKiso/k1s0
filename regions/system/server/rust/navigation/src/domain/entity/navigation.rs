use serde::{Deserialize, Serialize};

/// ナビゲーション設定のルート構造体。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NavigationConfig {
    pub version: u32,
    #[serde(default)]
    pub guards: Vec<Guard>,
    pub routes: Vec<Route>,
}

/// ルートガード定義。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Guard {
    pub id: String,
    #[serde(rename = "type")]
    pub guard_type: GuardType,
    pub redirect_to: String,
    #[serde(default)]
    pub roles: Vec<String>,
}

/// ルーティング定義。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Route {
    pub id: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub component_id: Option<String>,
    #[serde(default)]
    pub guards: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transition: Option<TransitionType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect_to: Option<String>,
    #[serde(default)]
    pub children: Vec<Route>,
    #[serde(default)]
    pub params: Vec<Param>,
}

/// ルートパラメータ定義。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Param {
    pub name: String,
    #[serde(rename = "type")]
    pub param_type: ParamType,
}

/// ガードの種別。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum GuardType {
    AuthRequired,
    RoleRequired,
    RedirectIfAuthenticated,
}

/// ページ遷移アニメーションの種別。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TransitionType {
    Fade,
    Slide,
    Modal,
}

/// ルートパラメータの型。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ParamType {
    String,
    Int,
    Uuid,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_navigation_yaml() {
        let yaml = r#"
version: 1
guards:
  - id: auth_required
    type: auth_required
    redirect_to: /login
  - id: admin_only
    type: role_required
    roles: [admin]
    redirect_to: /forbidden
routes:
  - id: root
    path: /
    redirect_to: /dashboard
  - id: login
    path: /login
    component_id: LoginPage
    guards: []
  - id: dashboard
    path: /dashboard
    component_id: DashboardPage
    guards: [auth_required]
    transition: fade
"#;
        let config: NavigationConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.version, 1);
        assert_eq!(config.guards.len(), 2);
        assert_eq!(config.guards[0].guard_type, GuardType::AuthRequired);
        assert_eq!(config.guards[1].guard_type, GuardType::RoleRequired);
        assert_eq!(config.guards[1].roles, vec!["admin"]);
        assert_eq!(config.routes.len(), 3);
        assert_eq!(config.routes[2].transition, Some(TransitionType::Fade));
    }

    #[test]
    fn deserialize_guard_types() {
        let yaml = r#"
id: test
type: redirect_if_authenticated
redirect_to: /dashboard
"#;
        let guard: Guard = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(guard.guard_type, GuardType::RedirectIfAuthenticated);
    }

    #[test]
    fn deserialize_route_with_params_and_children() {
        let yaml = r#"
id: users
path: /users
component_id: UsersPage
guards: [auth_required]
params:
  - name: id
    type: uuid
children:
  - id: user_detail
    path: ":id"
    component_id: UserDetailPage
    guards: [auth_required]
"#;
        let route: Route = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(route.params.len(), 1);
        assert_eq!(route.params[0].param_type, ParamType::Uuid);
        assert_eq!(route.children.len(), 1);
        assert_eq!(route.children[0].id, "user_detail");
    }
}
