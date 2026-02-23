use std::collections::HashMap;

use async_trait::async_trait;

use crate::domain::entity::secret::Secret;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait SecretStore: Send + Sync {
    async fn get(&self, path: &str, version: Option<i64>) -> anyhow::Result<Secret>;
    async fn set(&self, path: &str, data: HashMap<String, String>) -> anyhow::Result<i64>;
    async fn delete(&self, path: &str, versions: Vec<i64>) -> anyhow::Result<()>;
    async fn list(&self, path_prefix: &str) -> anyhow::Result<Vec<String>>;
    async fn exists(&self, path: &str) -> anyhow::Result<bool>;
}
