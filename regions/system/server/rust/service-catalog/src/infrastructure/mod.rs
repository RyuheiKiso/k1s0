pub mod config;
pub mod database;
pub mod health_collector;
pub mod startup;

use crate::domain::entity::claims::Claims;
use async_trait::async_trait;

/// `TokenVerifier` はトークン検証のためのトレイト。
/// JWKS エンドポイントから公開鍵を取得し、JWT の署名検証を行う。
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait TokenVerifier: Send + Sync {
    async fn verify_token(&self, token: &str) -> anyhow::Result<Claims>;
}

/// `JwksVerifierAdapter` は k1s0-auth ライブラリの `JwksVerifier` をラップするアダプター。
pub struct JwksVerifierAdapter {
    verifier: std::sync::Arc<k1s0_auth::JwksVerifier>,
}

impl JwksVerifierAdapter {
    #[must_use]
    pub fn new(verifier: std::sync::Arc<k1s0_auth::JwksVerifier>) -> Self {
        Self { verifier }
    }
}

#[async_trait]
impl TokenVerifier for JwksVerifierAdapter {
    async fn verify_token(&self, token: &str) -> anyhow::Result<Claims> {
        let lib_claims = self.verifier.verify_token(token).await?;
        // k1s0-auth の Claims を service-catalog の Claims に変換する
        Ok(Claims {
            sub: lib_claims.sub.clone(),
            iss: lib_claims.iss.clone(),
            aud: lib_claims.audience().unwrap_or_default().to_string(),
            // LOW-008: 安全な型変換（オーバーフロー防止）
            exp: i64::try_from(lib_claims.exp).unwrap_or(i64::MAX),
            iat: i64::try_from(lib_claims.iat).unwrap_or(0),
            jti: lib_claims.jti.clone().unwrap_or_default(),
            typ: lib_claims.typ.clone().unwrap_or_default(),
            azp: lib_claims.azp.clone().unwrap_or_default(),
            scope: lib_claims.scope.clone().unwrap_or_default(),
            preferred_username: lib_claims.preferred_username.clone().unwrap_or_default(),
            email: lib_claims.email.clone().unwrap_or_default(),
            realm_access: crate::domain::entity::claims::RealmAccess {
                roles: lib_claims.realm_roles().to_vec(),
            },
            resource_access: lib_claims
                .resource_access
                .as_ref()
                .map(|ra| {
                    ra.iter()
                        .map(|(k, v)| {
                            (
                                k.clone(),
                                crate::domain::entity::claims::ResourceAccess {
                                    roles: v.roles.clone(),
                                },
                            )
                        })
                        .collect()
                })
                .unwrap_or_default(),
            tier_access: lib_claims.tier_access_list().to_vec(),
            // tenant_id は Keycloak のカスタムクレームから取得する。JWT に含まれない場合は空文字列。
            tenant_id: lib_claims.tenant_id.clone(),
        })
    }
}
