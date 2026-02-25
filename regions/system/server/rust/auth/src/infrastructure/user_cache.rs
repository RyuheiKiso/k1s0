/// UserCache はユーザー情報のインメモリキャッシュ。
/// moka::future::Cache を使用し、TTL 付きでユーザー情報をキャッシュする。
use moka::future::Cache;
use std::sync::Arc;
use std::time::Duration;

use crate::domain::entity::user::User;

pub struct UserCache {
    inner: Cache<String, Arc<User>>,
}

impl UserCache {
    /// 新しい UserCache を作成する。
    ///
    /// # Arguments
    /// * `max_capacity` - キャッシュに保持する最大エントリ数
    /// * `ttl_secs` - エントリの有効期間（秒）
    pub fn new(max_capacity: u64, ttl_secs: u64) -> Self {
        let inner = Cache::builder()
            .max_capacity(max_capacity)
            .time_to_live(Duration::from_secs(ttl_secs))
            .build();
        Self { inner }
    }

    /// user_id に対応するユーザーを取得する。
    /// キャッシュミスの場合は None を返す。
    pub async fn get(&self, user_id: &str) -> Option<Arc<User>> {
        self.inner.get(user_id).await
    }

    /// ユーザーをキャッシュに追加する。
    pub async fn insert(&self, user_id: &str, user: &User) {
        self.inner
            .insert(user_id.to_string(), Arc::new(user.clone()))
            .await;
    }

    /// 特定のユーザーをキャッシュから削除する。
    pub async fn invalidate(&self, user_id: &str) {
        self.inner.invalidate(user_id).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

    #[tokio::test]
    async fn test_insert_and_get_returns_user() {
        let cache = UserCache::new(100, 60);
        let user = make_user("user-1");

        cache.insert("user-1", &user).await;

        let result = cache.get("user-1").await;
        assert!(result.is_some());
        let cached = result.unwrap();
        assert_eq!(cached.id, "user-1");
        assert_eq!(cached.username, "user-user-1");
    }

    #[tokio::test]
    async fn test_get_miss_returns_none() {
        let cache = UserCache::new(100, 60);

        let result = cache.get("nonexistent").await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_invalidate_removes_entry() {
        let cache = UserCache::new(100, 60);
        let user = make_user("user-1");
        cache.insert("user-1", &user).await;

        assert!(cache.get("user-1").await.is_some());

        cache.invalidate("user-1").await;

        let result = cache.get("user-1").await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_invalidate_does_not_affect_other_users() {
        let cache = UserCache::new(100, 60);
        let user1 = make_user("user-1");
        let user2 = make_user("user-2");
        cache.insert("user-1", &user1).await;
        cache.insert("user-2", &user2).await;

        cache.invalidate("user-1").await;

        assert!(cache.get("user-1").await.is_none());
        assert!(cache.get("user-2").await.is_some());
    }

    #[tokio::test]
    async fn test_insert_overwrites_existing_entry() {
        let cache = UserCache::new(100, 60);
        let user_v1 = make_user("user-1");
        cache.insert("user-1", &user_v1).await;

        let mut user_v2 = make_user("user-1");
        user_v2.username = "updated-user".to_string();
        cache.insert("user-1", &user_v2).await;

        let result = cache.get("user-1").await.unwrap();
        assert_eq!(result.username, "updated-user");
    }

    #[tokio::test]
    async fn test_ttl_expiry() {
        let cache = UserCache::new(100, 1);
        let user = make_user("user-1");
        cache.insert("user-1", &user).await;

        assert!(cache.get("user-1").await.is_some());

        tokio::time::sleep(Duration::from_millis(1200)).await;

        let result = cache.get("user-1").await;
        assert!(result.is_none());
    }
}
