use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entity::policy::Policy;
use crate::domain::repository::PolicyRepository;
use crate::infrastructure::cache::PolicyCache;

/// CachedPolicyRepository は PolicyCache を使ってキャッシュ付きの PolicyRepository を提供する。
/// 内部の delegate に対して読み取り時にキャッシュを挟み、書き込み時にキャッシュを無効化する。
pub struct CachedPolicyRepository {
    delegate: Arc<dyn PolicyRepository>,
    cache: Arc<PolicyCache>,
}

impl CachedPolicyRepository {
    pub fn new(delegate: Arc<dyn PolicyRepository>, cache: Arc<PolicyCache>) -> Self {
        Self { delegate, cache }
    }
}

#[async_trait]
impl PolicyRepository for CachedPolicyRepository {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<Policy>> {
        // キャッシュから取得を試みる
        if let Some(cached) = self.cache.get(id).await {
            return Ok(Some((*cached).clone()));
        }

        // キャッシュミス: delegate から取得してキャッシュに保存
        let result = self.delegate.find_by_id(id).await?;
        if let Some(ref policy) = result {
            self.cache.insert(Arc::new(policy.clone())).await;
        }
        Ok(result)
    }

    async fn find_all(&self) -> anyhow::Result<Vec<Policy>> {
        // find_all はキャッシュを通さない（全件取得はキャッシュ効率が悪いため）
        self.delegate.find_all().await
    }

    async fn create(&self, policy: &Policy) -> anyhow::Result<()> {
        self.delegate.create(policy).await?;
        // 作成したポリシーをキャッシュに追加
        self.cache.insert(Arc::new(policy.clone())).await;
        Ok(())
    }

    async fn update(&self, policy: &Policy) -> anyhow::Result<()> {
        self.delegate.update(policy).await?;
        // 更新後のポリシーでキャッシュを更新
        self.cache.insert(Arc::new(policy.clone())).await;
        Ok(())
    }

    async fn delete(&self, id: &Uuid) -> anyhow::Result<bool> {
        let deleted = self.delegate.delete(id).await?;
        if deleted {
            self.cache.invalidate(id).await;
        }
        Ok(deleted)
    }

    async fn exists_by_name(&self, name: &str) -> anyhow::Result<bool> {
        // exists_by_name はキャッシュを通さない（名前ベースの検索は ID キャッシュと合わない）
        self.delegate.exists_by_name(name).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::policy_repository::MockPolicyRepository;
    use chrono::Utc;
    use std::sync::atomic::{AtomicU32, Ordering};

    fn make_policy(id: Uuid) -> Policy {
        Policy {
            id,
            name: "test-policy".to_string(),
            description: "test".to_string(),
            rego_content: "package test".to_string(),
            package_path: String::new(),
            bundle_id: None,
            version: 1,
            enabled: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_find_by_id_cache_miss_then_hit() {
        let id = Uuid::new_v4();
        let _policy = make_policy(id);

        let call_count = Arc::new(AtomicU32::new(0));
        let call_count_clone = call_count.clone();

        let mut mock = MockPolicyRepository::new();
        mock.expect_find_by_id()
            .returning(move |_| {
                call_count_clone.fetch_add(1, Ordering::SeqCst);
                Ok(Some(make_policy(id)))
            });

        let cache = Arc::new(PolicyCache::new(100, 60));
        let cached_repo = CachedPolicyRepository::new(Arc::new(mock), cache);

        // 1回目: キャッシュミス → delegate 呼び出し
        let result = cached_repo.find_by_id(&id).await.unwrap();
        assert!(result.is_some());
        assert_eq!(call_count.load(Ordering::SeqCst), 1);

        // 2回目: キャッシュヒット → delegate 呼び出しなし
        let result = cached_repo.find_by_id(&id).await.unwrap();
        assert!(result.is_some());
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_create_populates_cache() {
        let id = Uuid::new_v4();
        let policy = make_policy(id);

        let mut mock = MockPolicyRepository::new();
        mock.expect_create().returning(|_| Ok(()));
        // find_by_id should NOT be called because cache is populated
        mock.expect_find_by_id().never();

        let cache = Arc::new(PolicyCache::new(100, 60));
        let cached_repo = CachedPolicyRepository::new(Arc::new(mock), cache.clone());

        cached_repo.create(&policy).await.unwrap();

        // キャッシュにあるのでdelegate不要
        let result = cached_repo.find_by_id(&id).await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "test-policy");
    }

    #[tokio::test]
    async fn test_delete_invalidates_cache() {
        let id = Uuid::new_v4();

        let call_count = Arc::new(AtomicU32::new(0));
        let call_count_clone = call_count.clone();

        let mut mock = MockPolicyRepository::new();
        mock.expect_create().returning(|_| Ok(()));
        mock.expect_delete().returning(|_| Ok(true));
        mock.expect_find_by_id()
            .returning(move |_| {
                call_count_clone.fetch_add(1, Ordering::SeqCst);
                Ok(None)
            });

        let cache = Arc::new(PolicyCache::new(100, 60));
        let cached_repo = CachedPolicyRepository::new(Arc::new(mock), cache);

        let policy = make_policy(id);
        cached_repo.create(&policy).await.unwrap();

        // 削除 → キャッシュも無効化
        cached_repo.delete(&id).await.unwrap();

        // 次回の find_by_id は delegate を呼ぶ
        let result = cached_repo.find_by_id(&id).await.unwrap();
        assert!(result.is_none());
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_update_refreshes_cache() {
        let id = Uuid::new_v4();
        let mut policy = make_policy(id);

        let mut mock = MockPolicyRepository::new();
        mock.expect_create().returning(|_| Ok(()));
        mock.expect_update().returning(|_| Ok(()));

        let cache = Arc::new(PolicyCache::new(100, 60));
        let cached_repo = CachedPolicyRepository::new(Arc::new(mock), cache);

        cached_repo.create(&policy).await.unwrap();

        // 更新
        policy.description = "updated".to_string();
        policy.version = 2;
        cached_repo.update(&policy).await.unwrap();

        // キャッシュからは更新後のデータが取れる
        let result = cached_repo.find_by_id(&id).await.unwrap().unwrap();
        assert_eq!(result.description, "updated");
        assert_eq!(result.version, 2);
    }
}
