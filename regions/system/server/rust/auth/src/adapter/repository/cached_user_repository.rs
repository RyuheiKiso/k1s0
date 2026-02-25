use std::sync::Arc;

use async_trait::async_trait;

use crate::domain::entity::user::{User, UserListResult, UserRoles};
use crate::domain::repository::UserRepository;
use crate::infrastructure::user_cache::UserCache;

/// CachedUserRepository は UserRepository をキャッシュでラップする。
/// find_by_id でキャッシュヒット時はDBアクセスをスキップする。
/// ユーザー情報は読み取り専用のため、書き込み系の invalidation は不要。
pub struct CachedUserRepository {
    inner: Arc<dyn UserRepository>,
    cache: Arc<UserCache>,
    metrics: Option<Arc<k1s0_telemetry::metrics::Metrics>>,
}

impl CachedUserRepository {
    /// 新しい CachedUserRepository を作成する。
    pub fn new(inner: Arc<dyn UserRepository>, cache: Arc<UserCache>) -> Self {
        Self {
            inner,
            cache,
            metrics: None,
        }
    }

    /// メトリクス付きの CachedUserRepository を作成する。
    pub fn with_metrics(
        inner: Arc<dyn UserRepository>,
        cache: Arc<UserCache>,
        metrics: Arc<k1s0_telemetry::metrics::Metrics>,
    ) -> Self {
        Self {
            inner,
            cache,
            metrics: Some(metrics),
        }
    }
}

#[async_trait]
impl UserRepository for CachedUserRepository {
    /// キャッシュヒット時はDBアクセスをスキップして即返却する。
    /// キャッシュミスの場合はDBから取得してキャッシュに格納してから返却する。
    async fn find_by_id(&self, user_id: &str) -> anyhow::Result<User> {
        // キャッシュヒット確認
        if let Some(cached) = self.cache.get(user_id).await {
            if let Some(ref m) = self.metrics {
                m.record_cache_hit("users");
            }
            return Ok((*cached).clone());
        }

        if let Some(ref m) = self.metrics {
            m.record_cache_miss("users");
        }

        // キャッシュミス: DBから取得
        let user = self.inner.find_by_id(user_id).await?;

        // キャッシュに格納
        self.cache.insert(user_id, &user).await;

        Ok(user)
    }

    /// list はキャッシュを使わず inner に委譲する。
    async fn list(
        &self,
        page: i32,
        page_size: i32,
        search: Option<String>,
        enabled: Option<bool>,
    ) -> anyhow::Result<UserListResult> {
        self.inner.list(page, page_size, search, enabled).await
    }

    /// get_roles はキャッシュを使わず inner に委譲する。
    async fn get_roles(&self, user_id: &str) -> anyhow::Result<UserRoles> {
        self.inner.get_roles(user_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::user::Pagination;
    use crate::domain::repository::user_repository::MockUserRepository;
    use chrono::Utc;
    use std::collections::HashMap;

    fn make_user(id: &str) -> User {
        User {
            id: id.to_string(),
            username: format!("user-{}", id),
            email: format!("{}@example.com", id),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            enabled: true,
            email_verified: true,
            created_at: Utc::now(),
            attributes: HashMap::new(),
        }
    }

    fn make_cache() -> Arc<UserCache> {
        Arc::new(UserCache::new(100, 60))
    }

    /// キャッシュヒット時はDBアクセスをスキップして即返却する。
    #[tokio::test]
    async fn test_cache_hit_skips_db() {
        let mut mock = MockUserRepository::new();
        // find_by_id が呼ばれてはいけない
        mock.expect_find_by_id().never();

        let cache = make_cache();
        let user = make_user("user-1");
        // 事前にキャッシュにエントリを挿入
        cache.insert("user-1", &user).await;

        let repo = CachedUserRepository::new(Arc::new(mock), cache);
        let result = repo.find_by_id("user-1").await.unwrap();

        assert_eq!(result.id, "user-1");
        assert_eq!(result.username, "user-user-1");
    }

    /// キャッシュミス時はDBから取得してキャッシュに格納する。
    #[tokio::test]
    async fn test_cache_miss_then_store() {
        let user = make_user("user-1");
        let user_clone = user.clone();

        let mut mock = MockUserRepository::new();
        mock.expect_find_by_id()
            .withf(|id| id == "user-1")
            .once()
            .returning(move |_| Ok(user_clone.clone()));

        let cache = make_cache();
        let repo = CachedUserRepository::new(Arc::new(mock), cache.clone());

        // 1回目: キャッシュミス → DBから取得
        let result = repo.find_by_id("user-1").await.unwrap();
        assert_eq!(result.id, "user-1");

        // キャッシュにエントリが格納されていることを確認
        let cached = cache.get("user-1").await;
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().id, "user-1");
    }

    /// list はキャッシュを使わず inner に委譲する。
    #[tokio::test]
    async fn test_list_delegates_to_inner() {
        let mut mock = MockUserRepository::new();
        mock.expect_list()
            .once()
            .returning(|page, page_size, _, _| {
                Ok(UserListResult {
                    users: vec![],
                    pagination: Pagination {
                        total_count: 0,
                        page,
                        page_size,
                        has_next: false,
                    },
                })
            });

        let cache = make_cache();
        let repo = CachedUserRepository::new(Arc::new(mock), cache);

        let result = repo.list(1, 20, None, None).await.unwrap();
        assert_eq!(result.pagination.page, 1);
        assert!(result.users.is_empty());
    }
}
