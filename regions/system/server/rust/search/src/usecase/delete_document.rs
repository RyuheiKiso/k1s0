use std::sync::Arc;

use crate::domain::repository::SearchRepository;

#[derive(Debug, Clone)]
pub struct DeleteDocumentInput {
    pub index_name: String,
    pub doc_id: String,
    /// テナント ID: CRIT-005 対応。RLS によるテナント分離のために使用する。
    pub tenant_id: String,
}

#[derive(Debug, thiserror::Error)]
pub enum DeleteDocumentError {
    #[error("document not found: {0}/{1}")]
    NotFound(String, String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct DeleteDocumentUseCase {
    repo: Arc<dyn SearchRepository>,
}

impl DeleteDocumentUseCase {
    pub fn new(repo: Arc<dyn SearchRepository>) -> Self {
        Self { repo }
    }

    /// CRIT-005 対応: `tenant_id` を渡して RLS セッション変数を設定してからドキュメントを削除する。
    pub async fn execute(&self, input: &DeleteDocumentInput) -> Result<bool, DeleteDocumentError> {
        let deleted = self
            .repo
            .delete_document(&input.index_name, &input.doc_id, &input.tenant_id)
            .await
            .map_err(|e| DeleteDocumentError::Internal(e.to_string()))?;

        if !deleted {
            return Err(DeleteDocumentError::NotFound(
                input.index_name.clone(),
                input.doc_id.clone(),
            ));
        }

        Ok(true)
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
        mock.expect_delete_document()
            .withf(|index, id, _tenant_id| index == "products" && id == "doc-1")
            .returning(|_, _, _| Ok(true));

        let uc = DeleteDocumentUseCase::new(Arc::new(mock));
        let input = DeleteDocumentInput {
            index_name: "products".to_string(),
            doc_id: "doc-1".to_string(),
            tenant_id: "tenant-a".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn not_found() {
        let mut mock = MockSearchRepository::new();
        mock.expect_delete_document().returning(|_, _, _| Ok(false));

        let uc = DeleteDocumentUseCase::new(Arc::new(mock));
        let input = DeleteDocumentInput {
            index_name: "products".to_string(),
            doc_id: "nonexistent".to_string(),
            tenant_id: "tenant-a".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            DeleteDocumentError::NotFound(index, id) => {
                assert_eq!(index, "products");
                assert_eq!(id, "nonexistent");
            }
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
