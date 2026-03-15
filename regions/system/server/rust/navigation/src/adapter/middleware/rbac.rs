//! RBAC ミドルウェア（Tower Layer / Service）。
//!
//! navigation-server の RBAC 対応表に基づくアクセス制御を提供する。
//!
//! | ロール          | リソース/アクション    |
//! |----------------|----------------------|
//! | sys_auditor+   | navigation/read      |
//! | sys_operator+  | navigation/write     |
//! | sys_admin のみ  | navigation/admin     |
//!
//! `AuthMiddlewareLayer` の後に配置し、`k1s0_auth::Claims` がリクエスト
//! エクステンションに存在することを前提とする。

use axum::{
    body::Body,
    extract::Request,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use k1s0_auth::Claims;

/// navigation-server の RBAC アクション定義。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationAction {
    /// 読み取り — sys_auditor 以上
    Read,
    /// 書き込み — sys_operator 以上
    Write,
    /// 管理 — sys_admin のみ
    Admin,
}

/// ロールの階層順序。上位ロールは下位ロールの権限を包含する。
///
/// sys_admin > sys_operator > sys_auditor
const ROLE_HIERARCHY: &[&str] = &["sys_auditor", "sys_operator", "sys_admin"];

/// 指定アクションに必要な最低ロールを返す。
fn minimum_role(action: NavigationAction) -> &'static str {
    match action {
        NavigationAction::Read => "sys_auditor",
        NavigationAction::Write => "sys_operator",
        NavigationAction::Admin => "sys_admin",
    }
}

/// ユーザーが指定アクションの実行権限を持つかを判定する。
///
/// ロール階層を考慮し、上位ロールは下位ロールの権限を包含する。
/// `sys_admin` は全権限を持つ。
fn has_navigation_permission(claims: &Claims, action: NavigationAction) -> bool {
    let min_role = minimum_role(action);
    let min_idx = ROLE_HIERARCHY
        .iter()
        .position(|r| *r == min_role)
        .unwrap_or(usize::MAX);

    let realm_roles = claims.realm_roles();
    realm_roles.iter().any(|user_role| {
        ROLE_HIERARCHY
            .iter()
            .position(|r| r == user_role)
            .map(|idx| idx >= min_idx)
            .unwrap_or(false)
    })
}

/// エラーレスポンスを生成するヘルパー。
fn forbidden_response(message: &str) -> Response {
    (
        StatusCode::FORBIDDEN,
        Json(json!({
            "error": "SYS_AUTH_FORBIDDEN",
            "message": message
        })),
    )
        .into_response()
}

fn unauthenticated_response() -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(json!({
            "error": "SYS_AUTH_UNAUTHENTICATED",
            "message": "認証が必要です"
        })),
    )
        .into_response()
}

// ---------------------------------------------------------------------------
// Tower Layer / Service
// ---------------------------------------------------------------------------

/// `RbacMiddlewareLayer` は指定された `NavigationAction` に対する RBAC チェックを行う Tower Layer。
///
/// `AuthMiddlewareLayer` の後に配置すること。
#[derive(Clone)]
pub struct RbacMiddlewareLayer {
    action: NavigationAction,
}

impl RbacMiddlewareLayer {
    pub fn new(action: NavigationAction) -> Self {
        Self { action }
    }

    /// 読み取り権限（sys_auditor 以上）を要求するレイヤーを生成する。
    pub fn read() -> Self {
        Self::new(NavigationAction::Read)
    }

    /// 書き込み権限（sys_operator 以上）を要求するレイヤーを生成する。
    pub fn write() -> Self {
        Self::new(NavigationAction::Write)
    }

    /// 管理権限（sys_admin のみ）を要求するレイヤーを生成する。
    pub fn admin() -> Self {
        Self::new(NavigationAction::Admin)
    }
}

impl<S> tower::Layer<S> for RbacMiddlewareLayer {
    type Service = RbacMiddlewareService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RbacMiddlewareService {
            inner,
            action: self.action,
        }
    }
}

/// `RbacMiddlewareService` はリクエストエクステンションの `Claims` を参照し、
/// ロール階層に基づくアクセス制御を行う Tower Service。
#[derive(Clone)]
pub struct RbacMiddlewareService<S> {
    inner: S,
    action: NavigationAction,
}

impl<S> tower::Service<Request<Body>> for RbacMiddlewareService<S>
where
    S: tower::Service<Request<Body>, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = Response;
    type Error = S::Error;
    type Future =
        std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, S::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let action = self.action;
        let mut inner = self.inner.clone();

        Box::pin(async move {
            // Claims がエクステンションに存在しない場合は未認証
            let claims = match req.extensions().get::<Claims>() {
                Some(c) => c.clone(),
                None => {
                    return Ok(unauthenticated_response());
                }
            };

            // RBAC チェック
            if !has_navigation_permission(&claims, action) {
                let min_role = minimum_role(action);
                tracing::warn!(
                    sub = %claims.sub,
                    required_role = %min_role,
                    action = ?action,
                    "RBAC: アクセス拒否"
                );
                return Ok(forbidden_response(&format!(
                    "この操作には {} 以上のロールが必要です",
                    min_role
                )));
            }

            inner.call(req).await
        })
    }
}

/// リクエストエクステンションから `Claims` を取得するヘルパー。
pub fn get_claims_from_request(req: &Request<Body>) -> Option<&Claims> {
    req.extensions().get::<Claims>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use k1s0_auth::{Audience, RealmAccess};

    fn make_claims(roles: Vec<&str>) -> Claims {
        Claims {
            sub: "user-1".into(),
            iss: "https://auth.example.com/realms/system".into(),
            aud: Audience(vec!["k1s0-system".into()]),
            exp: 9999999999,
            iat: 1000000000,
            jti: None,
            typ: None,
            azp: None,
            scope: None,
            preferred_username: Some("taro".into()),
            email: Some("taro@example.com".into()),
            realm_access: Some(RealmAccess {
                roles: roles.into_iter().map(String::from).collect(),
            }),
            resource_access: None,
            tier_access: None,
        }
    }

    // --- Read 権限 ---

    #[test]
    fn sys_auditor_can_read() {
        let claims = make_claims(vec!["sys_auditor"]);
        assert!(has_navigation_permission(&claims, NavigationAction::Read));
    }

    #[test]
    fn sys_operator_can_read() {
        let claims = make_claims(vec!["sys_operator"]);
        assert!(has_navigation_permission(&claims, NavigationAction::Read));
    }

    #[test]
    fn sys_admin_can_read() {
        let claims = make_claims(vec!["sys_admin"]);
        assert!(has_navigation_permission(&claims, NavigationAction::Read));
    }

    #[test]
    fn unprivileged_user_cannot_read() {
        let claims = make_claims(vec!["user"]);
        assert!(!has_navigation_permission(&claims, NavigationAction::Read));
    }

    // --- Write 権限 ---

    #[test]
    fn sys_auditor_cannot_write() {
        let claims = make_claims(vec!["sys_auditor"]);
        assert!(!has_navigation_permission(&claims, NavigationAction::Write));
    }

    #[test]
    fn sys_operator_can_write() {
        let claims = make_claims(vec!["sys_operator"]);
        assert!(has_navigation_permission(&claims, NavigationAction::Write));
    }

    #[test]
    fn sys_admin_can_write() {
        let claims = make_claims(vec!["sys_admin"]);
        assert!(has_navigation_permission(&claims, NavigationAction::Write));
    }

    // --- Admin 権限 ---

    #[test]
    fn sys_auditor_cannot_admin() {
        let claims = make_claims(vec!["sys_auditor"]);
        assert!(!has_navigation_permission(&claims, NavigationAction::Admin));
    }

    #[test]
    fn sys_operator_cannot_admin() {
        let claims = make_claims(vec!["sys_operator"]);
        assert!(!has_navigation_permission(&claims, NavigationAction::Admin));
    }

    #[test]
    fn sys_admin_can_admin() {
        let claims = make_claims(vec!["sys_admin"]);
        assert!(has_navigation_permission(&claims, NavigationAction::Admin));
    }

    // --- Edge cases ---

    #[test]
    fn no_roles_denied() {
        let claims = make_claims(vec![]);
        assert!(!has_navigation_permission(&claims, NavigationAction::Read));
        assert!(!has_navigation_permission(&claims, NavigationAction::Write));
        assert!(!has_navigation_permission(&claims, NavigationAction::Admin));
    }

    #[test]
    fn multiple_roles_highest_wins() {
        let claims = make_claims(vec!["sys_auditor", "sys_operator"]);
        assert!(has_navigation_permission(&claims, NavigationAction::Read));
        assert!(has_navigation_permission(&claims, NavigationAction::Write));
        assert!(!has_navigation_permission(&claims, NavigationAction::Admin));
    }
}
