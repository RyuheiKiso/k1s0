use moka::future::Cache;
use std::time::Duration;

#[derive(Clone)]
pub struct PermissionCache {
    cache: Cache<String, bool>,
}

impl PermissionCache {
    pub fn new(ttl_secs: u64, max_capacity: u64) -> Self {
        Self {
            cache: Cache::builder()
                .max_capacity(max_capacity)
                .time_to_live(Duration::from_secs(ttl_secs))
                .build(),
        }
    }

    pub async fn get(&self, key: &str) -> Option<bool> {
        self.cache.get(key).await
    }

    pub async fn insert(&self, key: String, allowed: bool) {
        self.cache.insert(key, allowed).await;
    }

    pub fn make_key(user_id: &str, resource: &str, action: &str) -> String {
        format!("{}:{}:{}", user_id, resource, action)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_permission_cache_insert_and_get() {
        let cache = PermissionCache::new(60, 100);
        let key = PermissionCache::make_key("user-1", "users", "read");

        assert!(cache.get(&key).await.is_none());

        cache.insert(key.clone(), true).await;
        assert_eq!(cache.get(&key).await, Some(true));
    }

    #[tokio::test]
    async fn test_permission_cache_denied() {
        let cache = PermissionCache::new(60, 100);
        let key = PermissionCache::make_key("user-2", "config", "delete");

        cache.insert(key.clone(), false).await;
        assert_eq!(cache.get(&key).await, Some(false));
    }

    #[test]
    fn test_make_key_format() {
        let key = PermissionCache::make_key("user-1", "users", "read");
        assert_eq!(key, "user-1:users:read");
    }
}
