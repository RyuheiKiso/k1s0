use std::sync::Arc;

use crate::domain::entity::search_index::SearchIndex;
use crate::domain::repository::SearchRepository;

#[derive(Debug, thiserror::Error)]
pub enum ListIndicesError {
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct ListIndicesUseCase {
    repo: Arc<dyn SearchRepository>,
}

impl ListIndicesUseCase {
    pub fn new(repo: Arc<dyn SearchRepository>) -> Self {
        Self { repo }
    }

    /// CRIT-005 対応: `tenant_id` を渡して RLS セッション変数を設定してからインデックス一覧を取得する。
    pub async fn execute(&self, tenant_id: &str) -> Result<Vec<SearchIndex>, ListIndicesError> {
        self.repo
            .list_indices(tenant_id)
            .await
            .map_err(|e| ListIndicesError::Internal(e.to_string()))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::repository::search_repository::MockSearchRepository;

    #[tokio::test]
    async fn success() {
        let mut mock = MockSearchRepository::new();
        mock.expect_list_indices().returning(|_| Ok(vec![]));

        let uc = ListIndicesUseCase::new(Arc::new(mock));
        let result = uc.execute("tenant-a").await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn internal_error() {
        let mut mock = MockSearchRepository::new();
        mock.expect_list_indices()
            .returning(|_| Err(anyhow::anyhow!("db error")));

        let uc = ListIndicesUseCase::new(Arc::new(mock));
        let result = uc.execute("tenant-a").await;
        assert!(result.is_err());
    }
}
