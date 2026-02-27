use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entity::api_key::ApiKey;

/// ApiKeyRepository は API キー管理のためのリポジトリトレイト。
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ApiKeyRepository: Send + Sync {
    /// API キーを作成する。
    async fn create(&self, api_key: &ApiKey) -> anyhow::Result<()>;

    /// ID で API キーを取得する。
    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<ApiKey>>;

    /// prefix で API キーを取得する（トークン検証用）。
    async fn find_by_prefix(&self, prefix: &str) -> anyhow::Result<Option<ApiKey>>;

    /// テナントの API キー一覧を取得する。
    async fn list_by_tenant(&self, tenant_id: &str) -> anyhow::Result<Vec<ApiKey>>;

    /// API キーを失効させる。
    async fn revoke(&self, id: Uuid) -> anyhow::Result<()>;

    /// API キーを削除する。
    async fn delete(&self, id: Uuid) -> anyhow::Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[tokio::test]
    async fn test_mock_api_key_repository_create() {
        let mut mock = MockApiKeyRepository::new();
        mock.expect_create().returning(|_| Ok(()));

        let now = Utc::now();
        let key = ApiKey {
            id: Uuid::new_v4(),
            tenant_id: "tenant-1".to_string(),
            name: "Test Key".to_string(),
            key_hash: "hash".to_string(),
            prefix: "k1s0_ab12".to_string(),
            scopes: vec!["read".to_string()],
            expires_at: None,
            revoked: false,
            created_at: now,
            updated_at: now,
        };

        let result = mock.create(&key).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mock_api_key_repository_find_by_id() {
        let mut mock = MockApiKeyRepository::new();
        let id = Uuid::new_v4();
        let expected_id = id;

        mock.expect_find_by_id()
            .withf(move |i| *i == expected_id)
            .returning(move |_| {
                let now = Utc::now();
                Ok(Some(ApiKey {
                    id,
                    tenant_id: "tenant-1".to_string(),
                    name: "Found Key".to_string(),
                    key_hash: "hash".to_string(),
                    prefix: "k1s0_ab12".to_string(),
                    scopes: vec!["read".to_string()],
                    expires_at: None,
                    revoked: false,
                    created_at: now,
                    updated_at: now,
                }))
            });

        let result = mock.find_by_id(expected_id).await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "Found Key");
    }

    #[tokio::test]
    async fn test_mock_api_key_repository_find_by_prefix() {
        let mut mock = MockApiKeyRepository::new();
        mock.expect_find_by_prefix()
            .withf(|p| p == "k1s0_ab12")
            .returning(|_| {
                let now = Utc::now();
                Ok(Some(ApiKey {
                    id: Uuid::new_v4(),
                    tenant_id: "tenant-1".to_string(),
                    name: "Prefix Key".to_string(),
                    key_hash: "hash".to_string(),
                    prefix: "k1s0_ab12".to_string(),
                    scopes: vec!["read".to_string()],
                    expires_at: None,
                    revoked: false,
                    created_at: now,
                    updated_at: now,
                }))
            });

        let result = mock.find_by_prefix("k1s0_ab12").await.unwrap();
        assert!(result.is_some());
    }

    #[tokio::test]
    async fn test_mock_api_key_repository_list_by_tenant() {
        let mut mock = MockApiKeyRepository::new();
        mock.expect_list_by_tenant()
            .withf(|t| t == "tenant-1")
            .returning(|_| Ok(vec![]));

        let result = mock.list_by_tenant("tenant-1").await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_mock_api_key_repository_revoke() {
        let mut mock = MockApiKeyRepository::new();
        mock.expect_revoke().returning(|_| Ok(()));

        let result = mock.revoke(Uuid::new_v4()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mock_api_key_repository_delete() {
        let mut mock = MockApiKeyRepository::new();
        mock.expect_delete().returning(|_| Ok(()));

        let result = mock.delete(Uuid::new_v4()).await;
        assert!(result.is_ok());
    }
}
