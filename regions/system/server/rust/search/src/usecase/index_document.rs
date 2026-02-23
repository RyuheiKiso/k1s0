use std::sync::Arc;

use crate::domain::entity::search_index::SearchDocument;
use crate::domain::repository::SearchRepository;

#[derive(Debug, Clone)]
pub struct IndexDocumentInput {
    pub id: String,
    pub index_name: String,
    pub content: serde_json::Value,
}

#[derive(Debug, thiserror::Error)]
pub enum IndexDocumentError {
    #[error("index not found: {0}")]
    IndexNotFound(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct IndexDocumentUseCase {
    repo: Arc<dyn SearchRepository>,
}

impl IndexDocumentUseCase {
    pub fn new(repo: Arc<dyn SearchRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, input: &IndexDocumentInput) -> Result<SearchDocument, IndexDocumentError> {
        let index = self
            .repo
            .find_index(&input.index_name)
            .await
            .map_err(|e| IndexDocumentError::Internal(e.to_string()))?;

        if index.is_none() {
            return Err(IndexDocumentError::IndexNotFound(input.index_name.clone()));
        }

        let doc = SearchDocument {
            id: input.id.clone(),
            index_name: input.index_name.clone(),
            content: input.content.clone(),
            indexed_at: chrono::Utc::now(),
        };

        self.repo
            .index_document(&doc)
            .await
            .map_err(|e| IndexDocumentError::Internal(e.to_string()))?;

        Ok(doc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::search_index::SearchIndex;
    use crate::domain::repository::search_repository::MockSearchRepository;

    #[tokio::test]
    async fn success() {
        let mut mock = MockSearchRepository::new();
        let index = SearchIndex::new("products".to_string(), serde_json::json!({}));
        let return_index = index.clone();

        mock.expect_find_index()
            .withf(|name| name == "products")
            .returning(move |_| Ok(Some(return_index.clone())));
        mock.expect_index_document().returning(|_| Ok(()));

        let uc = IndexDocumentUseCase::new(Arc::new(mock));
        let input = IndexDocumentInput {
            id: "doc-1".to_string(),
            index_name: "products".to_string(),
            content: serde_json::json!({"name": "Widget", "description": "A useful widget"}),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let doc = result.unwrap();
        assert_eq!(doc.id, "doc-1");
        assert_eq!(doc.index_name, "products");
    }

    #[tokio::test]
    async fn index_not_found() {
        let mut mock = MockSearchRepository::new();
        mock.expect_find_index().returning(|_| Ok(None));

        let uc = IndexDocumentUseCase::new(Arc::new(mock));
        let input = IndexDocumentInput {
            id: "doc-1".to_string(),
            index_name: "nonexistent".to_string(),
            content: serde_json::json!({}),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            IndexDocumentError::IndexNotFound(name) => assert_eq!(name, "nonexistent"),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
