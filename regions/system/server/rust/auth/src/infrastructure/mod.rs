pub mod database;
pub mod jwks_adapter;
pub mod jwks_provider;
pub mod kafka_producer;
pub mod keycloak_client;
pub mod permission_cache;
pub mod user_cache;

pub use jwks_adapter::JwksVerifierAdapter;

use crate::domain::entity::claims::Claims;
use async_trait::async_trait;

/// TokenVerifier はトークン検証のためのトレイト。
/// JWKS エンドポイントから公開鍵を取得し、JWT の署名検証を行う。
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait TokenVerifier: Send + Sync {
    async fn verify_token(&self, token: &str) -> anyhow::Result<Claims>;
}
