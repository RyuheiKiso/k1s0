pub mod database;
pub mod kafka_producer;
pub mod keycloak_client;

use async_trait::async_trait;
use crate::domain::entity::claims::Claims;

/// TokenVerifier はトークン検証のためのトレイト。
/// JWKS エンドポイントから公開鍵を取得し、JWT の署名検証を行う。
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait TokenVerifier: Send + Sync {
    async fn verify_token(&self, token: &str) -> anyhow::Result<Claims>;
}
