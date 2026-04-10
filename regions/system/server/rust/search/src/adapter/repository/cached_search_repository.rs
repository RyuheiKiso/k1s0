use std::sync::Arc;

use async_trait::async_trait;

use crate::domain::entity::search_index::{SearchDocument, SearchIndex, SearchQuery, SearchResult};
use crate::domain::repository::SearchRepository;
use crate::infrastructure::cache::IndexCache;

/// `CachedSearchRepository` は `IndexCache` を使ってキャッシュ付きの `SearchRepository` を提供する。
/// CRIT-005 対応: `tenant_id` を内部の delegate に透過的に渡す。
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
    /// CRIT-005 対応: `tenant_id` を delegate に渡し RLS を有効にしてインデックスを作成する。
    async fn create_index(&self, index: &SearchIndex, tenant_id: &str) -> anyhow::Result<()> {
        self.inner.create_index(index, tenant_id).await?;
        self.cache.insert(Arc::new(index.clone())).await;
        Ok(())
    }

    /// CRIT-005 対応: `tenant_id` を delegate に渡し RLS を有効にしてキャッシュ経由でインデックスを取得する。
    async fn find_index(&self, name: &str, tenant_id: &str) -> anyhow::Result<Option<SearchIndex>> {
        if let Some(cached) = self.cache.get(name).await {
            return Ok(Some((*cached).clone()));
        }

        let index = self.inner.find_index(name, tenant_id).await?;
        if let Some(ref idx) = index {
            self.cache.insert(Arc::new(idx.clone())).await;
        }
        Ok(index)
    }

    /// CRIT-005 対応: `tenant_id` を delegate に渡し RLS を有効にしてドキュメントを登録する。
    async fn index_document(&self, doc: &SearchDocument, tenant_id: &str) -> anyhow::Result<()> {
        self.inner.index_document(doc, tenant_id).await
    }

    /// CRIT-005 対応: `tenant_id` を delegate に渡し RLS を有効にして検索を実行する。
    async fn search(&self, query: &SearchQuery, tenant_id: &str) -> anyhow::Result<SearchResult> {
        self.inner.search(query, tenant_id).await
    }

    /// CRIT-005 対応: `tenant_id` を delegate に渡し RLS を有効にしてドキュメントを削除する。
    async fn delete_document(
        &self,
        index_name: &str,
        doc_id: &str,
        tenant_id: &str,
    ) -> anyhow::Result<bool> {
        self.inner
            .delete_document(index_name, doc_id, tenant_id)
            .await
    }

    /// CRIT-005 対応: `tenant_id` を delegate に渡し RLS を有効にして全インデックスを取得する。
    async fn list_indices(&self, tenant_id: &str) -> anyhow::Result<Vec<SearchIndex>> {
        let indices = self.inner.list_indices(tenant_id).await?;
        for idx in &indices {
            self.cache.insert(Arc::new(idx.clone())).await;
        }
        Ok(indices)
    }
}
