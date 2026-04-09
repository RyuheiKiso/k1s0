use std::sync::Arc;

use crate::domain::entity::search_index::SearchIndex;
use crate::domain::repository::SearchRepository;

#[derive(Debug, Clone)]
pub struct CreateIndexInput {
    pub name: String,
    pub mapping: serde_json::Value,
    /// テナント ID: CRIT-005 対応。RLS によるテナント分離のために使用する。
    pub tenant_id: String,
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

    /// CRIT-005 対応: `tenant_id` を渡して RLS セッション変数を設定してからインデックスを作成する。
    pub async fn execute(&self, input: &CreateIndexInput) -> Result<SearchIndex, CreateIndexError> {
        let existing = self
            .repo
            .find_index(&input.name, &input.tenant_id)
            .await
            .map_err(|e| CreateIndexError::Internal(e.to_string()))?;

        if existing.is_some() {
            return Err(CreateIndexError::AlreadyExists(input.name.clone()));
        }

        // CRIT-002 対応: SearchIndex::new() に tenant_id を渡し、ハードコードを排除する
        let index = SearchIndex::new(
            input.name.clone(),
            input.mapping.clone(),
            input.tenant_id.clone(),
        );

        self.repo
            .create_index(&index, &input.tenant_id)
            .await
            .map_err(|e| CreateIndexError::Internal(e.to_string()))?;

        Ok(index)
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
        mock.expect_find_index()
            .withf(|name, _tenant_id| name == "products")
            .returning(|_, _| Ok(None));
        mock.expect_create_index().returning(|_, _| Ok(()));

        let uc = CreateIndexUseCase::new(Arc::new(mock));
        let input = CreateIndexInput {
            name: "products".to_string(),
            mapping: serde_json::json!({"fields": ["name", "description"]}),
            tenant_id: "tenant-a".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let index = result.unwrap();
        assert_eq!(index.name, "products");
        assert_eq!(index.tenant_id, "tenant-a");
    }

    #[tokio::test]
    async fn already_exists() {
        let mut mock = MockSearchRepository::new();
        // テスト用のダミーインデックス（テナント IDは "tenant-a" を使用する）
        let existing = SearchIndex::new(
            "products".to_string(),
            serde_json::json!({}),
            "tenant-a".to_string(),
        );
        let return_index = existing.clone();
        mock.expect_find_index()
            .withf(|name, _tenant_id| name == "products")
            .returning(move |_, _| Ok(Some(return_index.clone())));

        let uc = CreateIndexUseCase::new(Arc::new(mock));
        let input = CreateIndexInput {
            name: "products".to_string(),
            mapping: serde_json::json!({}),
            tenant_id: "tenant-a".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            CreateIndexError::AlreadyExists(name) => assert_eq!(name, "products"),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
