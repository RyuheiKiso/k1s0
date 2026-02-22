use std::sync::Arc;

use async_trait::async_trait;

use crate::domain::entity::claims::{Claims, RealmAccess, ResourceAccess};
use crate::infrastructure::TokenVerifier;

/// JwksVerifierAdapter はライブラリの JwksVerifier をサーバーの TokenVerifier に適合させる。
pub struct JwksVerifierAdapter {
    verifier: Arc<k1s0_auth::JwksVerifier>,
}

impl JwksVerifierAdapter {
    pub fn new(verifier: Arc<k1s0_auth::JwksVerifier>) -> Self {
        Self { verifier }
    }
}

#[async_trait]
impl TokenVerifier for JwksVerifierAdapter {
    async fn verify_token(&self, token: &str) -> anyhow::Result<Claims> {
        let lib_claims = self
            .verifier
            .verify_token(token)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(convert_claims(lib_claims))
    }
}

fn convert_claims(c: k1s0_auth::Claims) -> Claims {
    let realm_access = c
        .realm_access
        .map(|ra| RealmAccess { roles: ra.roles })
        .unwrap_or_default();

    let resource_access = c
        .resource_access
        .map(|ra| {
            ra.into_iter()
                .map(|(k, v)| (k, ResourceAccess { roles: v.roles }))
                .collect()
        })
        .unwrap_or_default();

    Claims {
        sub: c.sub,
        iss: c.iss,
        aud: c.aud.0.first().cloned().unwrap_or_default(),
        exp: c.exp as i64,
        iat: c.iat as i64,
        jti: c.jti.unwrap_or_default(),
        typ: c.typ.unwrap_or_default(),
        azp: c.azp.unwrap_or_default(),
        scope: c.scope.unwrap_or_default(),
        preferred_username: c.preferred_username.unwrap_or_default(),
        email: c.email.unwrap_or_default(),
        realm_access,
        resource_access,
        tier_access: c.tier_access.unwrap_or_default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_convert_claims_full() {
        let lib_claims = k1s0_auth::Claims {
            sub: "user-uuid-1234".to_string(),
            iss: "https://auth.example.com/realms/k1s0".to_string(),
            aud: k1s0_auth::claims::Audience(vec!["k1s0-api".to_string(), "other-api".to_string()]),
            exp: 1710000900,
            iat: 1710000000,
            jti: Some("token-uuid-5678".to_string()),
            typ: Some("Bearer".to_string()),
            azp: Some("react-spa".to_string()),
            scope: Some("openid profile email".to_string()),
            preferred_username: Some("taro.yamada".to_string()),
            email: Some("taro.yamada@example.com".to_string()),
            realm_access: Some(k1s0_auth::claims::RealmAccess {
                roles: vec!["user".to_string(), "sys_admin".to_string()],
            }),
            resource_access: Some(HashMap::from([(
                "order-service".to_string(),
                k1s0_auth::claims::Access {
                    roles: vec!["read".to_string(), "write".to_string()],
                },
            )])),
            tier_access: Some(vec!["system".to_string(), "business".to_string()]),
        };

        let server_claims = convert_claims(lib_claims);

        assert_eq!(server_claims.sub, "user-uuid-1234");
        assert_eq!(server_claims.iss, "https://auth.example.com/realms/k1s0");
        assert_eq!(server_claims.aud, "k1s0-api");
        assert_eq!(server_claims.exp, 1710000900);
        assert_eq!(server_claims.iat, 1710000000);
        assert_eq!(server_claims.jti, "token-uuid-5678");
        assert_eq!(server_claims.typ, "Bearer");
        assert_eq!(server_claims.azp, "react-spa");
        assert_eq!(server_claims.scope, "openid profile email");
        assert_eq!(server_claims.preferred_username, "taro.yamada");
        assert_eq!(server_claims.email, "taro.yamada@example.com");
        assert_eq!(
            server_claims.realm_access.roles,
            vec!["user".to_string(), "sys_admin".to_string()]
        );
        assert_eq!(
            server_claims
                .resource_access
                .get("order-service")
                .unwrap()
                .roles,
            vec!["read".to_string(), "write".to_string()]
        );
        assert_eq!(
            server_claims.tier_access,
            vec!["system".to_string(), "business".to_string()]
        );
    }

    #[test]
    fn test_convert_claims_minimal() {
        let lib_claims = k1s0_auth::Claims {
            sub: "user-1".to_string(),
            iss: "issuer".to_string(),
            aud: k1s0_auth::claims::Audience(vec![]),
            exp: 100,
            iat: 50,
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

        let server_claims = convert_claims(lib_claims);

        assert_eq!(server_claims.sub, "user-1");
        assert_eq!(server_claims.iss, "issuer");
        assert_eq!(server_claims.aud, "");
        assert_eq!(server_claims.exp, 100);
        assert_eq!(server_claims.iat, 50);
        assert_eq!(server_claims.jti, "");
        assert_eq!(server_claims.typ, "");
        assert_eq!(server_claims.azp, "");
        assert_eq!(server_claims.scope, "");
        assert_eq!(server_claims.preferred_username, "");
        assert_eq!(server_claims.email, "");
        assert!(server_claims.realm_access.roles.is_empty());
        assert!(server_claims.resource_access.is_empty());
        assert!(server_claims.tier_access.is_empty());
    }
}
