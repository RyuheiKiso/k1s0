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

    pub async fn execute(&self) -> Result<Vec<SearchIndex>, ListIndicesError> {
        self.repo
            .list_indices()
            .await
            .map_err(|e| ListIndicesError::Internal(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::search_repository::MockSearchRepository;

    #[tokio::test]
    async fn success() {
        let mut mock = MockSearchRepository::new();
        mock.expect_list_indices().returning(|| Ok(vec![]));

        let uc = ListIndicesUseCase::new(Arc::new(mock));
        let result = uc.execute().await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn internal_error() {
        let mut mock = MockSearchRepository::new();
        mock.expect_list_indices()
            .returning(|| Err(anyhow::anyhow!("db error")));

        let uc = ListIndicesUseCase::new(Arc::new(mock));
        let result = uc.execute().await;
        assert!(result.is_err());
    }
}
