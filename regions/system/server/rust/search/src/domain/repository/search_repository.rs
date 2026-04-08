use async_trait::async_trait;

use crate::domain::entity::search_index::{SearchDocument, SearchIndex, SearchQuery, SearchResult};

/// CRIT-005 対応: 各メソッドに `tenant_id` を追加してテナント分離を実現する。
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait SearchRepository: Send + Sync {
    async fn create_index(&self, index: &SearchIndex, tenant_id: &str) -> anyhow::Result<()>;
    async fn find_index(&self, name: &str, tenant_id: &str) -> anyhow::Result<Option<SearchIndex>>;
    async fn index_document(&self, doc: &SearchDocument, tenant_id: &str) -> anyhow::Result<()>;
    async fn search(&self, query: &SearchQuery, tenant_id: &str) -> anyhow::Result<SearchResult>;
    async fn delete_document(&self, index_name: &str, doc_id: &str, tenant_id: &str) -> anyhow::Result<bool>;
    async fn list_indices(&self, tenant_id: &str) -> anyhow::Result<Vec<SearchIndex>>;
}
