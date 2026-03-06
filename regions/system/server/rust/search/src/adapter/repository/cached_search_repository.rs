use std::sync::Arc;

use async_trait::async_trait;

use crate::domain::entity::search_index::{SearchDocument, SearchIndex, SearchQuery, SearchResult};
use crate::domain::repository::SearchRepository;
use crate::infrastructure::cache::IndexCache;

pub struct CachedSearchRepository {
    inner: Arc<dyn SearchRepository>,
    cache: Arc<IndexCache>,
}

impl CachedSearchRepository {
    pub fn new(inner: Arc<dyn SearchRepository>, cache: Arc<IndexCache>) -> Self {
        Self { inner, cache }
    }
}

#[async_trait]
impl SearchRepository for CachedSearchRepository {
    async fn create_index(&self, index: &SearchIndex) -> anyhow::Result<()> {
        self.inner.create_index(index).await?;
        self.cache.insert(Arc::new(index.clone())).await;
        Ok(())
    }

    async fn find_index(&self, name: &str) -> anyhow::Result<Option<SearchIndex>> {
        if let Some(cached) = self.cache.get(name).await {
            return Ok(Some((*cached).clone()));
        }

        let index = self.inner.find_index(name).await?;
        if let Some(ref idx) = index {
            self.cache.insert(Arc::new(idx.clone())).await;
        }
        Ok(index)
    }

    async fn index_document(&self, doc: &SearchDocument) -> anyhow::Result<()> {
        self.inner.index_document(doc).await
    }

    async fn search(&self, query: &SearchQuery) -> anyhow::Result<SearchResult> {
        self.inner.search(query).await
    }

    async fn delete_document(&self, index_name: &str, doc_id: &str) -> anyhow::Result<bool> {
        self.inner.delete_document(index_name, doc_id).await
    }

    async fn list_indices(&self) -> anyhow::Result<Vec<SearchIndex>> {
        let indices = self.inner.list_indices().await?;
        for idx in &indices {
            self.cache.insert(Arc::new(idx.clone())).await;
        }
        Ok(indices)
    }
}
