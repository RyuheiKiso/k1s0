use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Claims は JWT トークンの Claims を表す。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default, utoipa::ToSchema)]
pub struct Claims {
    pub sub: String,
    pub iss: String,
    pub aud: String,
    pub exp: i64,
    pub iat: i64,
    #[serde(default)]
    pub jti: String,
    #[serde(default)]
    pub typ: String,
    #[serde(default)]
    pub azp: String,
    #[serde(default)]
    pub scope: String,
    #[serde(default)]
    pub preferred_username: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub realm_access: RealmAccess,
    #[serde(default)]
    pub resource_access: HashMap<String, ResourceAccess>,
    #[serde(default)]
    pub tier_access: Vec<String>,
}

/// RealmAccess は Keycloak の realm_access クレームを表す。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default, utoipa::ToSchema)]
pub struct RealmAccess {
    #[serde(default)]
    pub roles: Vec<String>,
}

/// ResourceAccess は Keycloak の resource_access クレームを表す。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default, utoipa::ToSchema)]
pub struct ResourceAccess {
    #[serde(default)]
    pub roles: Vec<String>,
}

impl Claims {
    /// ユーザーが指定されたレルムロールを持っているかどうかを判定する。
    pub fn has_realm_role(&self, role: &str) -> bool {
        self.realm_access.roles.iter().any(|r| r == role)
    }

    /// ユーザーが指定されたクライアントロールを持っているかどうかを判定する。
    pub fn has_client_role(&self, client: &str, role: &str) -> bool {
        self.resource_access
            .get(client)
            .map(|access| access.roles.iter().any(|r| r == role))
            .unwrap_or(false)
    }

    /// sys_operator 以上のロールを持っているかどうかを判定する。
    pub fn is_sys_operator_or_above(&self) -> bool {
        self.has_realm_role("sys_operator") || self.has_realm_role("sys_admin")
    }

    /// sys_auditor 以上のロールを持っているかどうかを判定する。
    pub fn is_sys_auditor_or_above(&self) -> bool {
        self.has_realm_role("sys_auditor")
            || self.has_realm_role("sys_operator")
            || self.has_realm_role("sys_admin")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_claims() -> Claims {
        Claims {
            sub: "user-uuid-1234".to_string(),
            iss: "https://auth.k1s0.internal.example.com/realms/k1s0".to_string(),
            aud: "k1s0-api".to_string(),
            exp: 1710000900,
            iat: 1710000000,
            jti: "token-uuid-5678".to_string(),
            typ: "Bearer".to_string(),
            azp: "react-spa".to_string(),
            scope: "openid profile email".to_string(),
            preferred_username: "taro.yamada".to_string(),
            email: "taro.yamada@example.com".to_string(),
            realm_access: RealmAccess {
                roles: vec!["user".to_string(), "sys_auditor".to_string()],
            },
            resource_access: HashMap::from([(
                "order-service".to_string(),
                ResourceAccess {
                    roles: vec!["read".to_string(), "write".to_string()],
                },
            )]),
            tier_access: vec![
                "system".to_string(),
                "business".to_string(),
                "service".to_string(),
            ],
        }
    }

    #[test]
    fn test_claims_has_realm_role() {
        let claims = sample_claims();
        assert!(claims.has_realm_role("user"));
        assert!(claims.has_realm_role("sys_auditor"));
        assert!(!claims.has_realm_role("sys_admin"));
    }

    #[test]
    fn test_claims_has_client_role() {
        let claims = sample_claims();
        assert!(claims.has_client_role("order-service", "read"));
        assert!(claims.has_client_role("order-service", "write"));
        assert!(!claims.has_client_role("order-service", "delete"));
        assert!(!claims.has_client_role("unknown-service", "read"));
    }

    #[test]
    fn test_is_sys_operator_or_above() {
        let mut claims = sample_claims();
        // has sys_auditor but not sys_operator
        assert!(!claims.is_sys_operator_or_above());

        claims.realm_access.roles.push("sys_operator".to_string());
        assert!(claims.is_sys_operator_or_above());
    }

    #[test]
    fn test_is_sys_auditor_or_above() {
        let claims = sample_claims();
        assert!(claims.is_sys_auditor_or_above());

        let minimal_claims = Claims {
            sub: "user-1".to_string(),
            iss: "iss".to_string(),
            aud: "aud".to_string(),
            exp: 0,
            iat: 0,
            realm_access: RealmAccess {
                roles: vec!["user".to_string()],
            },
            ..Default::default()
        };
        assert!(!minimal_claims.is_sys_auditor_or_above());
    }

    #[test]
    fn test_claims_serialization_roundtrip() {
        let claims = sample_claims();
        let json = serde_json::to_string(&claims).unwrap();
        let deserialized: Claims = serde_json::from_str(&json).unwrap();
        assert_eq!(claims, deserialized);
    }

    #[test]
    fn test_claims_default() {
        let claims = Claims::default();
        assert!(claims.sub.is_empty());
        assert!(claims.realm_access.roles.is_empty());
        assert!(claims.resource_access.is_empty());
        assert!(claims.tier_access.is_empty());
    }
}
