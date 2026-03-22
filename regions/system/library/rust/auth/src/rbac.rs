//! RBAC ヘルパー: ロール・パーミッション・Tier アクセスの判定。

use crate::claims::Claims;

/// Claims に指定のレルムロールが含まれるかを判定する。
pub fn has_role(claims: &Claims, role: &str) -> bool {
    claims.realm_roles().iter().any(|r| r == role)
}

/// Claims に指定のリソースロールが含まれるかを判定する。
pub fn has_resource_role(claims: &Claims, resource: &str, role: &str) -> bool {
    claims.resource_roles(resource).iter().any(|r| r == role)
}

/// Claims に指定の権限があるかを判定する。
///
/// realm_access と resource_access の両方をチェックする。
/// sys_admin のみ全リソース全アクションの権限を持つ（最小権限原則）。
/// admin ロールは realm_access に存在しても全権限を付与しない（Go 版と同一ロジック）。
/// resource_access に admin ロールがある場合はそのリソース内の全アクションを許可する。
pub fn check_permission(claims: &Claims, resource: &str, action: &str) -> bool {
    // sys_admin のみ全権限を付与する（スーパーユーザー）。
    // realm_access の admin ロールは通常ロールとして扱い、全権限を付与しない。
    if has_role(claims, "sys_admin") {
        return true;
    }

    // resource_access のチェック（指定リソースのロールを確認）。
    // resource_access に admin ロールがある場合はそのリソース内の全アクションを許可する。
    if let Some(ref resource_access) = claims.resource_access {
        if let Some(access) = resource_access.get(resource) {
            for role in &access.roles {
                if role == action || role == "admin" {
                    return true;
                }
            }
        }
    }

    false
}

/// Backward-compatible alias of `check_permission`.
pub fn has_permission(claims: &Claims, resource: &str, action: &str) -> bool {
    check_permission(claims, resource, action)
}

/// Tier の階層レベルを返す。
/// system(0) > business(1) > service(2) の順で上位 Tier ほど小さい値を返す。
/// 不明な Tier は None を返す。
fn tier_level(tier: &str) -> Option<u8> {
    match tier.to_ascii_lowercase().as_str() {
        "system" => Some(0),
        "business" => Some(1),
        "service" => Some(2),
        _ => None,
    }
}

/// Claims で指定 Tier へのアクセスが許可されているかを判定する。
///
/// Tier 階層ルール:
/// - system tier を持つユーザーは全 Tier (system, business, service) にアクセス可能
/// - business tier を持つユーザーは business と service にアクセス可能
/// - service tier を持つユーザーは service のみにアクセス可能
///
/// tier_access 配列内の各 Tier について、要求された Tier 以上の階層であればアクセスを許可する。
pub fn has_tier_access(claims: &Claims, required_tier: &str) -> bool {
    let required_level = match tier_level(required_tier) {
        Some(level) => level,
        None => return false,
    };

    claims.tier_access_list().iter().any(|user_tier| {
        tier_level(user_tier)
            .map(|user_level| user_level <= required_level)
            .unwrap_or(false)
    })
}

/// tier_access Claim を検証し、指定 Tier へのアクセス権がない場合はエラーを返す。
///
/// `has_tier_access` のラッパーで、ミドルウェアやユースケースから直接呼び出せる。
///
/// # Errors
///
/// 指定 Tier へのアクセス権がない場合は `AuthError::TierAccessDenied` を返す。
pub fn validate_tier_access(
    claims: &Claims,
    required_tier: &str,
) -> Result<(), crate::verifier::AuthError> {
    if has_tier_access(claims, required_tier) {
        Ok(())
    } else {
        Err(crate::verifier::AuthError::TierAccessDenied)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::claims::{Audience, RealmAccess, RoleSet};
    use std::collections::HashMap;

    fn make_claims(
        realm_roles: Vec<&str>,
        resource_access: HashMap<&str, Vec<&str>>,
        tier_access: Vec<&str>,
    ) -> Claims {
        Claims {
            sub: "user-1".into(),
            iss: "https://auth.example.com/realms/k1s0".into(),
            aud: Audience(vec!["k1s0-api".into()]),
            exp: 9999999999,
            iat: 1000000000,
            jti: None,
            typ: None,
            azp: None,
            scope: None,
            preferred_username: Some("taro".into()),
            email: Some("taro@example.com".into()),
            realm_access: Some(RealmAccess {
                roles: realm_roles.into_iter().map(String::from).collect(),
            }),
            resource_access: Some(
                resource_access
                    .into_iter()
                    .map(|(k, v)| {
                        (
                            k.to_string(),
                            RoleSet {
                                roles: v.into_iter().map(String::from).collect(),
                            },
                        )
                    })
                    .collect(),
            ),
            tier_access: Some(tier_access.into_iter().map(String::from).collect()),
        }
    }

    // has_role がロールの有無を正しく判定することを確認する。
    #[test]
    fn test_has_role() {
        let claims = make_claims(vec!["user", "order_manager"], HashMap::new(), vec![]);

        assert!(has_role(&claims, "user"));
        assert!(has_role(&claims, "order_manager"));
        assert!(!has_role(&claims, "admin"));
        assert!(!has_role(&claims, "sys_admin"));
    }

    // has_resource_role がリソースロールの有無を正しく判定することを確認する。
    #[test]
    fn test_has_resource_role() {
        let mut ra = HashMap::new();
        ra.insert("task-server", vec!["read", "write"]);
        let claims = make_claims(vec!["user"], ra, vec![]);

        assert!(has_resource_role(&claims, "task-server", "read"));
        assert!(has_resource_role(&claims, "task-server", "write"));
        assert!(!has_resource_role(&claims, "task-server", "delete"));
        assert!(!has_resource_role(&claims, "user-service", "read"));
    }

    // check_permission がリソースロールに基づいてアクセス可否を返すことを確認する。
    #[test]
    fn test_check_permission_basic() {
        let mut ra = HashMap::new();
        ra.insert("task-server", vec!["read", "write"]);
        let claims = make_claims(vec!["user"], ra, vec![]);

        assert!(check_permission(&claims, "task-server", "read"));
        assert!(check_permission(&claims, "task-server", "write"));
        assert!(!check_permission(&claims, "task-server", "delete"));
    }

    // has_permission が check_permission の別名として同じ結果を返すことを確認する。
    #[test]
    fn test_has_permission_alias() {
        let mut ra = HashMap::new();
        ra.insert("task-server", vec!["read"]);
        let claims = make_claims(vec!["user"], ra, vec![]);
        assert!(has_permission(&claims, "task-server", "read"));
        assert!(!has_permission(&claims, "task-server", "write"));
    }

    // sys_admin ロールを持つユーザーがすべてのリソースの全操作を許可されることを確認する。
    #[test]
    fn test_check_permission_sys_admin() {
        let claims = make_claims(vec!["sys_admin"], HashMap::new(), vec![]);

        assert!(check_permission(&claims, "any-resource", "read"));
        assert!(check_permission(&claims, "any-resource", "write"));
        assert!(check_permission(&claims, "any-resource", "delete"));
    }

    // realm_access の admin ロールは全権限を付与しないことを確認する（最小権限原則）。
    // Go 版 CheckPermission と同一ロジック: admin は resource_access のみで有効。
    #[test]
    fn test_check_permission_realm_admin_does_not_grant_all() {
        // realm_access に admin ロールがあっても全権限にはならない
        let claims = make_claims(vec!["admin"], HashMap::new(), vec![]);

        assert!(!check_permission(&claims, "task-server", "read"));
        assert!(!check_permission(&claims, "task-server", "write"));
        assert!(!check_permission(&claims, "any-resource", "delete"));
    }

    // リソース admin ロールを持つ場合はそのリソースへの全操作が許可されることを確認する。
    #[test]
    fn test_check_permission_resource_admin() {
        let mut ra = HashMap::new();
        ra.insert("task-server", vec!["admin"]);
        let claims = make_claims(vec!["user"], ra, vec![]);

        assert!(check_permission(&claims, "task-server", "read"));
        assert!(check_permission(&claims, "task-server", "write"));
        assert!(check_permission(&claims, "task-server", "delete"));
    }

    // system Tier を持つユーザーが全 Tier へのアクセスを許可されることを確認する。
    #[test]
    fn test_has_tier_access_system_grants_all() {
        let claims = make_claims(vec![], HashMap::new(), vec!["system"]);

        assert!(has_tier_access(&claims, "system"));
        assert!(has_tier_access(&claims, "business"));
        assert!(has_tier_access(&claims, "service"));
        assert!(has_tier_access(&claims, "System")); // case insensitive
    }

    // business Tier を持つユーザーが business と service のみアクセスできることを確認する。
    #[test]
    fn test_has_tier_access_business_grants_business_and_service() {
        let claims = make_claims(vec![], HashMap::new(), vec!["business"]);

        assert!(!has_tier_access(&claims, "system"));
        assert!(has_tier_access(&claims, "business"));
        assert!(has_tier_access(&claims, "service"));
        assert!(has_tier_access(&claims, "Business")); // case insensitive
    }

    // service Tier を持つユーザーが service のみアクセスできることを確認する。
    #[test]
    fn test_has_tier_access_service_grants_service_only() {
        let claims = make_claims(vec![], HashMap::new(), vec!["service"]);

        assert!(!has_tier_access(&claims, "system"));
        assert!(!has_tier_access(&claims, "business"));
        assert!(has_tier_access(&claims, "service"));
        assert!(has_tier_access(&claims, "Service")); // case insensitive
    }

    // 複数の Tier を持つ場合にそれぞれの Tier へのアクセスが許可されることを確認する。
    #[test]
    fn test_has_tier_access_multiple_tiers() {
        let claims = make_claims(vec![], HashMap::new(), vec!["business", "service"]);

        assert!(!has_tier_access(&claims, "system"));
        assert!(has_tier_access(&claims, "business"));
        assert!(has_tier_access(&claims, "service"));
    }

    // tier_access が None の場合にすべての Tier へのアクセスが拒否されることを確認する。
    #[test]
    fn test_has_tier_access_empty() {
        let claims = Claims {
            sub: "user-1".into(),
            iss: "iss".into(),
            aud: Audience(vec![]),
            exp: 9999999999,
            iat: 1000000000,
            jti: None,
            typ: None,
            azp: None,
            scope: None,
            preferred_username: None,
            email: None,
            realm_access: None,
            resource_access: None,
            tier_access: None,
        };

        assert!(!has_tier_access(&claims, "system"));
        assert!(!has_tier_access(&claims, "business"));
        assert!(!has_tier_access(&claims, "service"));
    }

    // 不明な required_tier を指定した場合に has_tier_access が false を返すことを確認する。
    #[test]
    fn test_has_tier_access_unknown_required_tier() {
        let claims = make_claims(vec![], HashMap::new(), vec!["system"]);

        assert!(!has_tier_access(&claims, "unknown"));
    }

    // 不明なユーザー Tier を持つ場合にすべての Tier へのアクセスが拒否されることを確認する。
    #[test]
    fn test_has_tier_access_unknown_user_tier() {
        let claims = make_claims(vec![], HashMap::new(), vec!["unknown"]);

        assert!(!has_tier_access(&claims, "system"));
        assert!(!has_tier_access(&claims, "business"));
        assert!(!has_tier_access(&claims, "service"));
    }

    // 許可された Tier に対して validate_tier_access が Ok を返すことを確認する。
    #[test]
    fn test_validate_tier_access_ok() {
        use crate::rbac::validate_tier_access;

        let claims = make_claims(vec![], HashMap::new(), vec!["system"]);
        assert!(validate_tier_access(&claims, "system").is_ok());
        assert!(validate_tier_access(&claims, "business").is_ok());
        assert!(validate_tier_access(&claims, "service").is_ok());
    }

    // 許可されていない Tier に対して validate_tier_access がエラーを返すことを確認する。
    #[test]
    fn test_validate_tier_access_denied() {
        use crate::rbac::validate_tier_access;

        let claims = make_claims(vec![], HashMap::new(), vec!["service"]);
        assert!(validate_tier_access(&claims, "system").is_err());
        assert!(validate_tier_access(&claims, "business").is_err());
        assert!(validate_tier_access(&claims, "service").is_ok());
    }

    // tier_access が None の場合にすべての Tier で validate_tier_access がエラーを返すことを確認する。
    #[test]
    fn test_validate_tier_access_empty_denied() {
        use crate::rbac::validate_tier_access;

        let claims = Claims {
            sub: "user-1".into(),
            iss: "iss".into(),
            aud: Audience(vec![]),
            exp: 9999999999,
            iat: 1000000000,
            jti: None,
            typ: None,
            azp: None,
            scope: None,
            preferred_username: None,
            email: None,
            realm_access: None,
            resource_access: None,
            tier_access: None,
        };

        assert!(validate_tier_access(&claims, "system").is_err());
        assert!(validate_tier_access(&claims, "business").is_err());
        assert!(validate_tier_access(&claims, "service").is_err());
    }
}
