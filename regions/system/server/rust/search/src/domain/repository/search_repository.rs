use async_trait::async_trait;

use crate::domain::entity::search_index::{SearchDocument, SearchIndex, SearchQuery, SearchResult};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait SearchRepository: Send + Sync {
    async fn create_index(&self, index: &SearchIndex) -> anyhow::Result<()>;
    async fn find_index(&self, name: &str) -> anyhow::Result<Option<SearchIndex>>;
    async fn index_document(&self, doc: &SearchDocument) -> anyhow::Result<()>;
    async fn search(&self, query: &SearchQuery) -> anyhow::Result<SearchResult>;
    async fn delete_document(&self, index_name: &str, doc_id: &str) -> anyhow::Result<bool>;
    async fn list_indices(&self) -> anyhow::Result<Vec<SearchIndex>>;
}
