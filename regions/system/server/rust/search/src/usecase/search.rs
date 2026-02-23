use std::sync::Arc;

use crate::domain::entity::search_index::{SearchQuery, SearchResult};
use crate::domain::repository::SearchRepository;

#[derive(Debug, Clone)]
pub struct SearchInput {
    pub index_name: String,
    pub query: String,
    pub from: u32,
    pub size: u32,
}

#[derive(Debug, thiserror::Error)]
pub enum SearchError {
    #[error("index not found: {0}")]
    IndexNotFound(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct SearchUseCase {
    repo: Arc<dyn SearchRepository>,
}

impl SearchUseCase {
    pub fn new(repo: Arc<dyn SearchRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, input: &SearchInput) -> Result<SearchResult, SearchError> {
        let index = self
            .repo
            .find_index(&input.index_name)
            .await
            .map_err(|e| SearchError::Internal(e.to_string()))?;

        if index.is_none() {
            return Err(SearchError::IndexNotFound(input.index_name.clone()));
        }

        let query = SearchQuery {
            index_name: input.index_name.clone(),
            query: input.query.clone(),
            from: input.from,
            size: input.size,
        };

        self.repo
            .search(&query)
            .await
            .map_err(|e| SearchError::Internal(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::search_index::{SearchDocument, SearchIndex, SearchResult};
    use crate::domain::repository::search_repository::MockSearchRepository;

    #[tokio::test]
    async fn success() {
        let mut mock = MockSearchRepository::new();
        let index = SearchIndex::new("products".to_string(), serde_json::json!({}));
        let return_index = index.clone();

        mock.expect_find_index()
            .withf(|name| name == "products")
            .returning(move |_| Ok(Some(return_index.clone())));

        mock.expect_search().returning(|_| {
            Ok(SearchResult {
                total: 1,
                hits: vec![SearchDocument {
                    id: "doc-1".to_string(),
                    index_name: "products".to_string(),
                    content: serde_json::json!({"name": "Widget"}),
                    indexed_at: chrono::Utc::now(),
                }],
            })
        });

        let uc = SearchUseCase::new(Arc::new(mock));
        let input = SearchInput {
            index_name: "products".to_string(),
            query: "Widget".to_string(),
            from: 0,
            size: 10,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let search_result = result.unwrap();
        assert_eq!(search_result.total, 1);
        assert_eq!(search_result.hits.len(), 1);
        assert_eq!(search_result.hits[0].id, "doc-1");
    }

    #[tokio::test]
    async fn index_not_found() {
        let mut mock = MockSearchRepository::new();
        mock.expect_find_index().returning(|_| Ok(None));

        let uc = SearchUseCase::new(Arc::new(mock));
        let input = SearchInput {
            index_name: "nonexistent".to_string(),
            query: "test".to_string(),
            from: 0,
            size: 10,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            SearchError::IndexNotFound(name) => assert_eq!(name, "nonexistent"),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
