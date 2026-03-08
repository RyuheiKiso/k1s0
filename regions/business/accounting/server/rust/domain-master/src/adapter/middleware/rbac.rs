//! RBAC（ロールベースアクセス制御）ミドルウェア。
//!
//! domain-master サーバー固有のロール定義と権限チェックを提供する。
//!
//! ## ロール階層
//!
//! | ロール | read | write | admin |
//! |--------|------|-------|-------|
//! | biz_auditor | YES | - | - |
//! | biz_operator | YES | YES | - |
//! | biz_admin | YES | YES | YES |
//!
//! 上位ロールは下位の権限を包含する。

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use k1s0_auth::Claims;

/// domain-master で認識するビジネス層ロール。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BusinessRole {
    /// 読み取り専用。
    BizAuditor = 0,
    /// 読み書き。
    BizOperator = 1,
    /// 全権限（削除含む）。
    BizAdmin = 2,
}

impl BusinessRole {
    /// ロール文字列からパースする。不明なロールは `None` を返す。
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "biz_auditor" => Some(Self::BizAuditor),
            "biz_operator" => Some(Self::BizOperator),
            "biz_admin" => Some(Self::BizAdmin),
            _ => None,
        }
    }

    /// ロール名を返す。
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::BizAuditor => "biz_auditor",
            Self::BizOperator => "biz_operator",
            Self::BizAdmin => "biz_admin",
        }
    }
}

/// API アクション種別。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// 読み取り操作（GET）。biz_auditor 以上が必要。
    Read,
    /// 書き込み操作（POST / PUT）。biz_operator 以上が必要。
    Write,
    /// 管理操作（DELETE）。biz_admin のみ。
    Admin,
}

impl Action {
    /// アクションに必要な最低ロールを返す。
    pub fn minimum_role(&self) -> BusinessRole {
        match self {
            Action::Read => BusinessRole::BizAuditor,
            Action::Write => BusinessRole::BizOperator,
            Action::Admin => BusinessRole::BizAdmin,
        }
    }
}

/// ユーザーの Claims からビジネスロールの最上位を取得する。
pub fn highest_role(claims: &Claims) -> Option<BusinessRole> {
    claims
        .realm_access
        .as_ref()
        .and_then(|access| {
            access
                .roles
                .iter()
                .filter_map(|r| BusinessRole::from_str(r))
                .max()
        })
}

/// 指定アクションに対して十分なロールを持っているか判定する。
pub fn has_permission(claims: &Claims, action: Action) -> bool {
    match highest_role(claims) {
        Some(role) => role >= action.minimum_role(),
        None => false,
    }
}

/// RBAC ミドルウェアファクトリ。
///
/// 指定されたアクションに対する権限チェックを行うミドルウェアを返す。
/// Claims が Extension に挿入されていない場合（未認証）は 401 を返す。
/// 権限不足の場合は 403 を返す。
pub async fn rbac_middleware(req: Request, next: Next, action: Action) -> Response {
    let claims = match req.extensions().get::<Claims>() {
        Some(c) => c.clone(),
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "code": "BIZ_AUTH_UNAUTHENTICATED",
                    "message": "Authentication required"
                })),
            )
                .into_response();
        }
    };

    if !has_permission(&claims, action) {
        let required = action.minimum_role();
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "code": "BIZ_AUTH_PERMISSION_DENIED",
                "message": format!(
                    "Permission denied: '{}' role or higher is required for this operation",
                    required.as_str()
                )
            })),
        )
            .into_response();
    }

    next.run(req).await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_claims_with_roles(roles: Vec<&str>) -> Claims {
        Claims {
            sub: "test-user".to_string(),
            iss: "test".to_string(),
            aud: k1s0_auth::Audience(vec![]),
            exp: 0,
            iat: 0,
            jti: None,
            typ: None,
            azp: None,
            scope: None,
            preferred_username: None,
            email: None,
            realm_access: Some(k1s0_auth::RealmAccess {
                roles: roles.into_iter().map(String::from).collect(),
            }),
            resource_access: None,
            tier_access: None,
        }
    }

    #[test]
    fn test_business_role_ordering() {
        assert!(BusinessRole::BizAdmin > BusinessRole::BizOperator);
        assert!(BusinessRole::BizOperator > BusinessRole::BizAuditor);
    }

    #[test]
    fn test_business_role_from_str() {
        assert_eq!(
            BusinessRole::from_str("biz_auditor"),
            Some(BusinessRole::BizAuditor)
        );
        assert_eq!(
            BusinessRole::from_str("biz_operator"),
            Some(BusinessRole::BizOperator)
        );
        assert_eq!(
            BusinessRole::from_str("biz_admin"),
            Some(BusinessRole::BizAdmin)
        );
        assert_eq!(BusinessRole::from_str("unknown"), None);
    }

    #[test]
    fn test_highest_role_picks_max() {
        let claims = make_claims_with_roles(vec!["biz_auditor", "biz_operator"]);
        assert_eq!(highest_role(&claims), Some(BusinessRole::BizOperator));
    }

    #[test]
    fn test_highest_role_admin() {
        let claims = make_claims_with_roles(vec!["biz_admin", "biz_auditor"]);
        assert_eq!(highest_role(&claims), Some(BusinessRole::BizAdmin));
    }

    #[test]
    fn test_highest_role_no_biz_roles() {
        let claims = make_claims_with_roles(vec!["other_role"]);
        assert_eq!(highest_role(&claims), None);
    }

    #[test]
    fn test_has_permission_auditor_read() {
        let claims = make_claims_with_roles(vec!["biz_auditor"]);
        assert!(has_permission(&claims, Action::Read));
        assert!(!has_permission(&claims, Action::Write));
        assert!(!has_permission(&claims, Action::Admin));
    }

    #[test]
    fn test_has_permission_operator_read_write() {
        let claims = make_claims_with_roles(vec!["biz_operator"]);
        assert!(has_permission(&claims, Action::Read));
        assert!(has_permission(&claims, Action::Write));
        assert!(!has_permission(&claims, Action::Admin));
    }

    #[test]
    fn test_has_permission_admin_all() {
        let claims = make_claims_with_roles(vec!["biz_admin"]);
        assert!(has_permission(&claims, Action::Read));
        assert!(has_permission(&claims, Action::Write));
        assert!(has_permission(&claims, Action::Admin));
    }

    #[test]
    fn test_has_permission_no_roles() {
        let claims = make_claims_with_roles(vec![]);
        assert!(!has_permission(&claims, Action::Read));
    }

    #[test]
    fn test_action_minimum_role() {
        assert_eq!(Action::Read.minimum_role(), BusinessRole::BizAuditor);
        assert_eq!(Action::Write.minimum_role(), BusinessRole::BizOperator);
        assert_eq!(Action::Admin.minimum_role(), BusinessRole::BizAdmin);
    }
}
