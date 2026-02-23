use async_trait::async_trait;

use crate::domain::entity::api_registration::{ApiSchema, ApiSchemaVersion};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ApiSchemaRepository: Send + Sync {
    async fn find_by_name(&self, name: &str) -> anyhow::Result<Option<ApiSchema>>;
    async fn find_all(
        &self,
        schema_type: Option<String>,
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<(Vec<ApiSchema>, u64)>;
    async fn create(&self, schema: &ApiSchema) -> anyhow::Result<()>;
    async fn update(&self, schema: &ApiSchema) -> anyhow::Result<()>;
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ApiSchemaVersionRepository: Send + Sync {
    async fn find_by_name_and_version(
        &self,
        name: &str,
        version: u32,
    ) -> anyhow::Result<Option<ApiSchemaVersion>>;
    async fn find_latest_by_name(&self, name: &str) -> anyhow::Result<Option<ApiSchemaVersion>>;
    async fn find_all_by_name(
        &self,
        name: &str,
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<(Vec<ApiSchemaVersion>, u64)>;
    async fn create(&self, version: &ApiSchemaVersion) -> anyhow::Result<()>;
    async fn delete(&self, name: &str, version: u32) -> anyhow::Result<bool>;
    async fn count_by_name(&self, name: &str) -> anyhow::Result<u64>;
}
