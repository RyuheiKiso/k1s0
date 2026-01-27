//! 統合テスト
//!
//! testcontainers を使用した Redis 統合テスト。

#[cfg(all(test, feature = "redis"))]
mod redis_tests {
    use k1s0_cache::{CacheClient, CacheConfig, CacheOperations};
    use std::time::Duration;
    use testcontainers::{runners::AsyncRunner, ContainerAsync};
    use testcontainers_modules::redis::Redis;

    async fn setup_redis() -> (ContainerAsync<Redis>, CacheClient) {
        let container = Redis::default().start().await.unwrap();
        let host_port = container.get_host_port_ipv4(6379).await.unwrap();

        let config = CacheConfig::builder()
            .host("127.0.0.1")
            .port(host_port)
            .key_prefix("test")
            .build()
            .unwrap();

        let client = CacheClient::new(config).await.unwrap();

        (container, client)
    }

    #[tokio::test]
    async fn test_basic_operations() {
        let (_container, client) = setup_redis().await;

        // Set
        client
            .set("key1", &"value1", Some(Duration::from_secs(60)))
            .await
            .unwrap();

        // Get
        let value: Option<String> = client.get("key1").await.unwrap();
        assert_eq!(value, Some("value1".to_string()));

        // Exists
        assert!(client.exists("key1").await.unwrap());
        assert!(!client.exists("nonexistent").await.unwrap());

        // Delete
        assert!(client.delete("key1").await.unwrap());
        assert!(!client.exists("key1").await.unwrap());
    }

    #[tokio::test]
    async fn test_serialization() {
        let (_container, client) = setup_redis().await;

        #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
        struct User {
            id: u64,
            name: String,
            email: String,
        }

        let user = User {
            id: 123,
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
        };

        client
            .set("user:123", &user, Some(Duration::from_secs(60)))
            .await
            .unwrap();

        let retrieved: Option<User> = client.get("user:123").await.unwrap();
        assert_eq!(retrieved, Some(user));
    }

    #[tokio::test]
    async fn test_mget_mset() {
        let (_container, client) = setup_redis().await;

        let items: Vec<(&str, &i32)> = vec![("a", &1), ("b", &2), ("c", &3)];

        client
            .mset(&items, Some(Duration::from_secs(60)))
            .await
            .unwrap();

        let values: Vec<Option<i32>> = client.mget(&["a", "b", "c", "d"]).await.unwrap();
        assert_eq!(values, vec![Some(1), Some(2), Some(3), None]);
    }

    #[tokio::test]
    async fn test_incr_decr() {
        let (_container, client) = setup_redis().await;

        // Incr on non-existent
        let val = client.incr("counter", 1).await.unwrap();
        assert_eq!(val, 1);

        // Incr existing
        let val = client.incr("counter", 5).await.unwrap();
        assert_eq!(val, 6);

        // Decr
        let val = client.decr("counter", 2).await.unwrap();
        assert_eq!(val, 4);
    }

    #[tokio::test]
    async fn test_set_nx() {
        let (_container, client) = setup_redis().await;

        // First set succeeds
        assert!(client
            .set_nx("unique", &"first", Some(Duration::from_secs(60)))
            .await
            .unwrap());

        // Second set fails
        assert!(!client
            .set_nx("unique", &"second", Some(Duration::from_secs(60)))
            .await
            .unwrap());

        // Value is still first
        let value: Option<String> = client.get("unique").await.unwrap();
        assert_eq!(value, Some("first".to_string()));
    }

    #[tokio::test]
    async fn test_ttl_and_expire() {
        let (_container, client) = setup_redis().await;

        client
            .set("expiring", &"value", Some(Duration::from_secs(100)))
            .await
            .unwrap();

        let ttl = client.ttl("expiring").await.unwrap();
        assert!(ttl.is_some());
        assert!(ttl.unwrap().as_secs() > 90);

        // Update TTL
        client
            .expire("expiring", Duration::from_secs(200))
            .await
            .unwrap();

        let ttl = client.ttl("expiring").await.unwrap();
        assert!(ttl.unwrap().as_secs() > 190);
    }

    #[tokio::test]
    async fn test_get_or_set() {
        let (_container, client) = setup_redis().await;
        use std::sync::atomic::{AtomicU32, Ordering};
        use std::sync::Arc;

        let call_count = Arc::new(AtomicU32::new(0));

        // First call - executes loader
        let count = call_count.clone();
        let value: i32 = client
            .get_or_set(
                "computed",
                || async move {
                    count.fetch_add(1, Ordering::SeqCst);
                    Ok(42)
                },
                Some(Duration::from_secs(60)),
            )
            .await
            .unwrap();

        assert_eq!(value, 42);
        assert_eq!(call_count.load(Ordering::SeqCst), 1);

        // Second call - cache hit
        let count = call_count.clone();
        let value: i32 = client
            .get_or_set(
                "computed",
                || async move {
                    count.fetch_add(1, Ordering::SeqCst);
                    Ok(100)
                },
                Some(Duration::from_secs(60)),
            )
            .await
            .unwrap();

        assert_eq!(value, 42);
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_metrics() {
        let (_container, client) = setup_redis().await;

        // Some operations
        client
            .set("metrics_test", &"value", Some(Duration::from_secs(60)))
            .await
            .unwrap();
        let _: Option<String> = client.get("metrics_test").await.unwrap();
        let _: Option<String> = client.get("nonexistent").await.unwrap();

        let metrics = client.metrics();
        assert!(metrics.hits() >= 1);
        assert!(metrics.misses() >= 1);
        assert!(metrics.operations() >= 3);
    }

    #[tokio::test]
    async fn test_pool_status() {
        let (_container, client) = setup_redis().await;

        let status = client.pool_status();
        assert!(status.max_connections > 0);
    }

    #[tokio::test]
    async fn test_hash_operations() {
        use k1s0_cache::HashOperations;

        let (_container, client) = setup_redis().await;

        // hset
        client.hset("hash1", "field1", &"value1").await.unwrap();
        client.hset("hash1", "field2", &"value2").await.unwrap();

        // hget
        let value: Option<String> = client.hget("hash1", "field1").await.unwrap();
        assert_eq!(value, Some("value1".to_string()));

        // hexists
        assert!(client.hexists("hash1", "field1").await.unwrap());
        assert!(!client.hexists("hash1", "nonexistent").await.unwrap());

        // hlen
        assert_eq!(client.hlen("hash1").await.unwrap(), 2);

        // hgetall
        let all: std::collections::HashMap<String, String> =
            client.hgetall("hash1").await.unwrap();
        assert_eq!(all.len(), 2);
        assert_eq!(all.get("field1"), Some(&"value1".to_string()));

        // hdel
        assert_eq!(client.hdel("hash1", &["field1"]).await.unwrap(), 1);
        assert_eq!(client.hlen("hash1").await.unwrap(), 1);
    }

    #[tokio::test]
    async fn test_list_operations() {
        use k1s0_cache::ListOperations;

        let (_container, client) = setup_redis().await;

        // lpush/rpush
        client.lpush("list1", &[1, 2, 3]).await.unwrap();
        client.rpush("list1", &[4, 5]).await.unwrap();

        // llen
        assert_eq!(client.llen("list1").await.unwrap(), 5);

        // lrange
        let values: Vec<i32> = client.lrange("list1", 0, -1).await.unwrap();
        assert_eq!(values, vec![3, 2, 1, 4, 5]);

        // lpop/rpop
        let value: Option<i32> = client.lpop("list1").await.unwrap();
        assert_eq!(value, Some(3));

        let value: Option<i32> = client.rpop("list1").await.unwrap();
        assert_eq!(value, Some(5));
    }

    #[tokio::test]
    async fn test_set_operations() {
        use k1s0_cache::SetOperations;

        let (_container, client) = setup_redis().await;

        // sadd
        client.sadd("set1", &["a", "b", "c"]).await.unwrap();

        // scard
        assert_eq!(client.scard("set1").await.unwrap(), 3);

        // sismember
        assert!(client.sismember("set1", &"a").await.unwrap());
        assert!(!client.sismember("set1", &"d").await.unwrap());

        // smembers
        let members: Vec<String> = client.smembers("set1").await.unwrap();
        assert_eq!(members.len(), 3);

        // srem
        assert_eq!(client.srem("set1", &["a"]).await.unwrap(), 1);
        assert_eq!(client.scard("set1").await.unwrap(), 2);
    }

    #[tokio::test]
    async fn test_key_prefix() {
        let (_container, client) = setup_redis().await;

        // Key should be prefixed
        client
            .set("mykey", &"value", Some(Duration::from_secs(60)))
            .await
            .unwrap();

        // The actual key in Redis is "test:mykey"
        let value: Option<String> = client.get("mykey").await.unwrap();
        assert_eq!(value, Some("value".to_string()));
    }
}

#[cfg(all(test, feature = "redis"))]
mod pattern_tests {
    use k1s0_cache::patterns::{CacheAside, CacheAsideConfig, TtlRefresh, TtlRefreshConfig};
    use k1s0_cache::{CacheClient, CacheConfig};
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;
    use std::time::Duration;
    use testcontainers::{runners::AsyncRunner, ContainerAsync};
    use testcontainers_modules::redis::Redis;

    async fn setup_redis() -> (ContainerAsync<Redis>, Arc<CacheClient>) {
        let container = Redis::default().start().await.unwrap();
        let host_port = container.get_host_port_ipv4(6379).await.unwrap();

        let config = CacheConfig::builder()
            .host("127.0.0.1")
            .port(host_port)
            .build()
            .unwrap();

        let client = Arc::new(CacheClient::new(config).await.unwrap());

        (container, client)
    }

    #[tokio::test]
    async fn test_cache_aside_pattern() {
        let (_container, client) = setup_redis().await;

        let config = CacheAsideConfig::default().with_default_ttl(Duration::from_secs(60));

        let cache_aside = CacheAside::new(client.clone(), config);

        let load_count = Arc::new(AtomicU32::new(0));

        // First call - cache miss
        let count = load_count.clone();
        let value: String = cache_aside
            .get_or_load("user:123", || async move {
                count.fetch_add(1, Ordering::SeqCst);
                Ok("Alice".to_string())
            })
            .await
            .unwrap();

        assert_eq!(value, "Alice");
        assert_eq!(load_count.load(Ordering::SeqCst), 1);

        // Second call - cache hit
        let count = load_count.clone();
        let value: String = cache_aside
            .get_or_load("user:123", || async move {
                count.fetch_add(1, Ordering::SeqCst);
                Ok("Bob".to_string())
            })
            .await
            .unwrap();

        assert_eq!(value, "Alice");
        assert_eq!(load_count.load(Ordering::SeqCst), 1);

        // Invalidate
        assert!(cache_aside.invalidate("user:123").await.unwrap());

        // Third call - cache miss again
        let count = load_count.clone();
        let value: String = cache_aside
            .get_or_load("user:123", || async move {
                count.fetch_add(1, Ordering::SeqCst);
                Ok("Charlie".to_string())
            })
            .await
            .unwrap();

        assert_eq!(value, "Charlie");
        assert_eq!(load_count.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_ttl_refresh_pattern() {
        let (_container, client) = setup_redis().await;

        let config = TtlRefreshConfig::default()
            .with_initial_ttl(Duration::from_secs(100))
            .with_refresh_ttl(Duration::from_secs(200));

        let ttl_refresh = TtlRefresh::new(client.clone(), config);

        // Load value
        let value: String = ttl_refresh
            .get_or_load_with_refresh("session:abc", || async { Ok("session_data".to_string()) })
            .await
            .unwrap();

        assert_eq!(value, "session_data");

        // Explicit refresh
        assert!(ttl_refresh.refresh("session:abc").await.unwrap());
    }
}
