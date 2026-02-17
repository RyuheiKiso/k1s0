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
/// admin ロール（sys_admin, admin, リソース admin）を持つ場合は全権限を付与する。
pub fn has_permission(claims: &Claims, resource: &str, action: &str) -> bool {
    // sys_admin は全権限
    if has_role(claims, "sys_admin") {
        return true;
    }

    // realm_access に admin ロールがある場合
    if has_role(claims, "admin") {
        return true;
    }

    // resource_access のチェック
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

/// Claims で指定 Tier へのアクセスが許可されているかを判定する。
pub fn has_tier_access(claims: &Claims, tier: &str) -> bool {
    claims
        .tier_access_list()
        .iter()
        .any(|t| t.eq_ignore_ascii_case(tier))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::claims::{Access, Audience, RealmAccess};
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
                            Access {
                                roles: v.into_iter().map(String::from).collect(),
                            },
                        )
                    })
                    .collect(),
            ),
            tier_access: Some(tier_access.into_iter().map(String::from).collect()),
        }
    }

    #[test]
    fn test_has_role() {
        let claims = make_claims(vec!["user", "order_manager"], HashMap::new(), vec![]);

        assert!(has_role(&claims, "user"));
        assert!(has_role(&claims, "order_manager"));
        assert!(!has_role(&claims, "admin"));
        assert!(!has_role(&claims, "sys_admin"));
    }

    #[test]
    fn test_has_resource_role() {
        let mut ra = HashMap::new();
        ra.insert("order-service", vec!["read", "write"]);
        let claims = make_claims(vec!["user"], ra, vec![]);

        assert!(has_resource_role(&claims, "order-service", "read"));
        assert!(has_resource_role(&claims, "order-service", "write"));
        assert!(!has_resource_role(&claims, "order-service", "delete"));
        assert!(!has_resource_role(&claims, "user-service", "read"));
    }

    #[test]
    fn test_has_permission_basic() {
        let mut ra = HashMap::new();
        ra.insert("order-service", vec!["read", "write"]);
        let claims = make_claims(vec!["user"], ra, vec![]);

        assert!(has_permission(&claims, "order-service", "read"));
        assert!(has_permission(&claims, "order-service", "write"));
        assert!(!has_permission(&claims, "order-service", "delete"));
    }

    #[test]
    fn test_has_permission_sys_admin() {
        let claims = make_claims(vec!["sys_admin"], HashMap::new(), vec![]);

        assert!(has_permission(&claims, "any-resource", "read"));
        assert!(has_permission(&claims, "any-resource", "write"));
        assert!(has_permission(&claims, "any-resource", "delete"));
    }

    #[test]
    fn test_has_permission_resource_admin() {
        let mut ra = HashMap::new();
        ra.insert("order-service", vec!["admin"]);
        let claims = make_claims(vec!["user"], ra, vec![]);

        assert!(has_permission(&claims, "order-service", "read"));
        assert!(has_permission(&claims, "order-service", "write"));
        assert!(has_permission(&claims, "order-service", "delete"));
    }

    #[test]
    fn test_has_tier_access() {
        let claims = make_claims(vec![], HashMap::new(), vec!["system", "business"]);

        assert!(has_tier_access(&claims, "system"));
        assert!(has_tier_access(&claims, "business"));
        assert!(has_tier_access(&claims, "System")); // case insensitive
        assert!(!has_tier_access(&claims, "service"));
    }

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
    }
}
