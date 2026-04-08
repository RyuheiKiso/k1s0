use std::sync::Arc;

use crate::domain::entity::search_index::SearchDocument;
use crate::domain::repository::SearchRepository;
use crate::infrastructure::kafka_producer::{DocumentIndexedEvent, SearchEventPublisher};

#[derive(Debug, Clone)]
pub struct IndexDocumentInput {
    pub id: String,
    pub index_name: String,
    pub content: serde_json::Value,
    /// テナント ID: CRIT-005 対応。RLS によるテナント分離のために使用する。
    pub tenant_id: String,
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
    event_publisher: Arc<dyn SearchEventPublisher>,
}

impl IndexDocumentUseCase {
    pub fn new(
        repo: Arc<dyn SearchRepository>,
        event_publisher: Arc<dyn SearchEventPublisher>,
    ) -> Self {
        Self {
            repo,
            event_publisher,
        }
    }

    /// CRIT-005 対応: `tenant_id` を渡して RLS セッション変数を設定してからドキュメントを登録する。
    pub async fn execute(
        &self,
        input: &IndexDocumentInput,
    ) -> Result<SearchDocument, IndexDocumentError> {
        let index = self
            .repo
            .find_index(&input.index_name, &input.tenant_id)
            .await
            .map_err(|e| IndexDocumentError::Internal(e.to_string()))?;

        if index.is_none() {
            return Err(IndexDocumentError::IndexNotFound(input.index_name.clone()));
        }

        let doc = SearchDocument {
            id: input.id.clone(),
            index_name: input.index_name.clone(),
            content: input.content.clone(),
            score: 0.0,
            indexed_at: chrono::Utc::now(),
        };

        self.repo
            .index_document(&doc, &input.tenant_id)
            .await
            .map_err(|e| IndexDocumentError::Internal(e.to_string()))?;

        self.event_publisher
            .publish_document_indexed(&DocumentIndexedEvent {
                event_type: "DOCUMENT_INDEXED".to_string(),
                index_name: doc.index_name.clone(),
                document_id: doc.id.clone(),
                actor_user_id: None,
                before: None,
                after: serde_json::json!({
                    "index_name": doc.index_name.clone(),
                    "document_id": doc.id.clone()
                }),
                timestamp: chrono::Utc::now().to_rfc3339(),
            })
            .await
            .map_err(|e| IndexDocumentError::Internal(e.to_string()))?;

        Ok(doc)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::entity::search_index::SearchIndex;
    use crate::domain::repository::search_repository::MockSearchRepository;
    use crate::infrastructure::kafka_producer::MockSearchEventPublisher;

    #[tokio::test]
    async fn success() {
        let mut mock = MockSearchRepository::new();
        // テスト用のダミーインデックス（テナント IDは "tenant-a" を使用する）
        let index = SearchIndex::new("products".to_string(), serde_json::json!({}), "tenant-a".to_string());
        let return_index = index.clone();

        mock.expect_find_index()
            .withf(|name, _tenant_id| name == "products")
            .returning(move |_, _| Ok(Some(return_index.clone())));
        mock.expect_index_document().returning(|_, _| Ok(()));
        let mut mock_publisher = MockSearchEventPublisher::new();
        mock_publisher
            .expect_publish_document_indexed()
            .returning(|_| Ok(()));

        let uc = IndexDocumentUseCase::new(Arc::new(mock), Arc::new(mock_publisher));
        let input = IndexDocumentInput {
            id: "doc-1".to_string(),
            index_name: "products".to_string(),
            content: serde_json::json!({"name": "Widget", "description": "A useful widget"}),
            tenant_id: "tenant-a".to_string(),
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
        mock.expect_find_index().returning(|_, _| Ok(None));

        let uc = IndexDocumentUseCase::new(
            Arc::new(mock),
            Arc::new(crate::infrastructure::kafka_producer::NoopSearchEventPublisher),
        );
        let input = IndexDocumentInput {
            id: "doc-1".to_string(),
            index_name: "nonexistent".to_string(),
            content: serde_json::json!({}),
            tenant_id: "tenant-a".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            IndexDocumentError::IndexNotFound(name) => assert_eq!(name, "nonexistent"),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
