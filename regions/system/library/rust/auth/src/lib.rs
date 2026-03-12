//! k1s0-auth: サーバー用 JWT JWKS 検証 + RBAC ライブラリ
//!
//! JWKS エンドポイントから公開鍵を取得し、JWT の署名検証を行う。
//! Keycloak が発行する JWT Claims に準拠した認証・認可チェックを提供する。
//!
//! # 使い方
//!
//! ```ignore
//! use k1s0_auth::JwksVerifier;
//! use std::time::Duration;
//!
//! let verifier = JwksVerifier::new(
//!     "https://auth.example.com/realms/k1s0/protocol/openid-connect/certs",
//!     "https://auth.example.com/realms/k1s0",
//!     "k1s0-api",
//!     Duration::from_secs(600),
//! );
//!
//! let claims = verifier.verify_token("eyJ...").await?;
//! ```

pub mod claims;
pub mod device_flow;
pub mod middleware;
pub mod rbac;
pub mod verifier;

pub use claims::{actor_from_claims, Audience, Claims, RealmAccess, RoleSet};
pub use device_flow::{DeviceAuthClient, DeviceCodeResponse, DeviceFlowError, TokenResult};
pub use middleware::{
    auth_middleware, get_claims, require_permission, require_role, require_tier_access,
};
pub use rbac::{
    check_permission, has_permission, has_resource_role, has_role, has_tier_access,
    validate_tier_access,
};
pub use verifier::{AuthError, DefaultJwksFetcher, JwksFetcher, JwksVerifier};

#[cfg(test)]
mod tests;
