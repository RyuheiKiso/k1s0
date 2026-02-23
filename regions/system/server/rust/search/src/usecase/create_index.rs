use std::sync::Arc;

use crate::domain::entity::search_index::SearchIndex;
use crate::domain::repository::SearchRepository;

#[derive(Debug, Clone)]
pub struct CreateIndexInput {
    pub name: String,
    pub mapping: serde_json::Value,
}

#[derive(Debug, thiserror::Error)]
pub enum CreateIndexError {
    #[error("index already exists: {0}")]
    AlreadyExists(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct CreateIndexUseCase {
    repo: Arc<dyn SearchRepository>,
}

impl CreateIndexUseCase {
    pub fn new(repo: Arc<dyn SearchRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, input: &CreateIndexInput) -> Result<SearchIndex, CreateIndexError> {
        let existing = self
            .repo
            .find_index(&input.name)
            .await
            .map_err(|e| CreateIndexError::Internal(e.to_string()))?;

        if existing.is_some() {
            return Err(CreateIndexError::AlreadyExists(input.name.clone()));
        }

        let index = SearchIndex::new(input.name.clone(), input.mapping.clone());

        self.repo
            .create_index(&index)
            .await
            .map_err(|e| CreateIndexError::Internal(e.to_string()))?;

        Ok(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::search_repository::MockSearchRepository;

    #[tokio::test]
    async fn success() {
        let mut mock = MockSearchRepository::new();
        mock.expect_find_index()
            .withf(|name| name == "products")
            .returning(|_| Ok(None));
        mock.expect_create_index().returning(|_| Ok(()));

        let uc = CreateIndexUseCase::new(Arc::new(mock));
        let input = CreateIndexInput {
            name: "products".to_string(),
            mapping: serde_json::json!({"fields": ["name", "description"]}),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let index = result.unwrap();
        assert_eq!(index.name, "products");
    }

    #[tokio::test]
    async fn already_exists() {
        let mut mock = MockSearchRepository::new();
        let existing = SearchIndex::new(
            "products".to_string(),
            serde_json::json!({}),
        );
        let return_index = existing.clone();
        mock.expect_find_index()
            .withf(|name| name == "products")
            .returning(move |_| Ok(Some(return_index.clone())));

        let uc = CreateIndexUseCase::new(Arc::new(mock));
        let input = CreateIndexInput {
            name: "products".to_string(),
            mapping: serde_json::json!({}),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            CreateIndexError::AlreadyExists(name) => assert_eq!(name, "products"),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
