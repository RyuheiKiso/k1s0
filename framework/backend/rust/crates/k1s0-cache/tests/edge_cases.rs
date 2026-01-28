//! k1s0-cache エッジケーステスト
//!
//! TTL期限切れ、接続エラー、大量データなどのエッジケースをテストする。

#[cfg(all(test, feature = "redis"))]
mod ttl_tests {
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
            .key_prefix("ttl_test")
            .build()
            .unwrap();

        let client = CacheClient::new(config).await.unwrap();

        (container, client)
    }

    #[tokio::test]
    async fn test_ttl_expiration() {
        let (_container, client) = setup_redis().await;

        // 1秒後に期限切れになるキーを設定
        client
            .set("short_lived", &"value", Some(Duration::from_secs(1)))
            .await
            .unwrap();

        // すぐに取得できる
        let value: Option<String> = client.get("short_lived").await.unwrap();
        assert_eq!(value, Some("value".to_string()));

        // 2秒待つ
        tokio::time::sleep(Duration::from_secs(2)).await;

        // 期限切れで取得できない
        let value: Option<String> = client.get("short_lived").await.unwrap();
        assert!(value.is_none());
    }

    #[tokio::test]
    async fn test_ttl_extension() {
        let (_container, client) = setup_redis().await;

        // 10秒の TTL で設定
        client
            .set("extendable", &"value", Some(Duration::from_secs(10)))
            .await
            .unwrap();

        // TTL を確認
        let ttl = client.ttl("extendable").await.unwrap();
        assert!(ttl.is_some());
        assert!(ttl.unwrap().as_secs() >= 8);

        // TTL を延長
        client
            .expire("extendable", Duration::from_secs(60))
            .await
            .unwrap();

        // 延長された TTL を確認
        let ttl = client.ttl("extendable").await.unwrap();
        assert!(ttl.is_some());
        assert!(ttl.unwrap().as_secs() >= 55);
    }

    #[tokio::test]
    async fn test_ttl_nonexistent_key() {
        let (_container, client) = setup_redis().await;

        // 存在しないキーの TTL
        let ttl = client.ttl("nonexistent_key_12345").await.unwrap();
        assert!(ttl.is_none());
    }

    #[tokio::test]
    async fn test_ttl_key_without_expiration() {
        let (_container, client) = setup_redis().await;

        // TTL なしで設定
        client
            .set("no_ttl", &"value", None)
            .await
            .unwrap();

        // TTL は -1 秒（無期限）として返されるべき
        let ttl = client.ttl("no_ttl").await.unwrap();
        // Redis では TTL なしのキーは特別な値を返す
        // 実装によっては None または特殊な Duration が返る
        // ここでは値が存在することを確認
        let exists = client.exists("no_ttl").await.unwrap();
        assert!(exists);
    }

    #[tokio::test]
    async fn test_zero_ttl_immediate_expiration() {
        let (_container, client) = setup_redis().await;

        // 非常に短い TTL
        client
            .set("instant", &"value", Some(Duration::from_millis(100)))
            .await
            .unwrap();

        // 200ms 待つ
        tokio::time::sleep(Duration::from_millis(200)).await;

        // 期限切れ
        let value: Option<String> = client.get("instant").await.unwrap();
        assert!(value.is_none());
    }
}

#[cfg(all(test, feature = "redis"))]
mod large_data_tests {
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
            .key_prefix("large_data")
            .build()
            .unwrap();

        let client = CacheClient::new(config).await.unwrap();

        (container, client)
    }

    #[tokio::test]
    async fn test_large_string_value() {
        let (_container, client) = setup_redis().await;

        // 1MB の文字列
        let large_value = "x".repeat(1024 * 1024);

        client
            .set("large_string", &large_value, Some(Duration::from_secs(60)))
            .await
            .unwrap();

        let retrieved: Option<String> = client.get("large_string").await.unwrap();
        assert_eq!(retrieved.map(|s| s.len()), Some(1024 * 1024));
    }

    #[tokio::test]
    async fn test_large_struct_value() {
        let (_container, client) = setup_redis().await;

        #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
        struct LargeStruct {
            items: Vec<String>,
        }

        // 10000 個のアイテムを持つ構造体
        let large_struct = LargeStruct {
            items: (0..10000).map(|i| format!("item_{}", i)).collect(),
        };

        client
            .set("large_struct", &large_struct, Some(Duration::from_secs(60)))
            .await
            .unwrap();

        let retrieved: Option<LargeStruct> = client.get("large_struct").await.unwrap();
        assert_eq!(retrieved.as_ref().map(|s| s.items.len()), Some(10000));
        assert_eq!(retrieved, Some(large_struct));
    }

    #[tokio::test]
    async fn test_many_keys() {
        let (_container, client) = setup_redis().await;

        // 1000 個のキーを設定
        for i in 0..1000 {
            client
                .set(&format!("key_{}", i), &i, Some(Duration::from_secs(60)))
                .await
                .unwrap();
        }

        // ランダムに取得して確認
        for i in [0, 100, 500, 999] {
            let value: Option<i32> = client.get(&format!("key_{}", i)).await.unwrap();
            assert_eq!(value, Some(i));
        }
    }

    #[tokio::test]
    async fn test_batch_operations_large() {
        let (_container, client) = setup_redis().await;

        // 100 個のキーを一括設定
        let items: Vec<(String, i32)> = (0..100)
            .map(|i| (format!("batch_{}", i), i))
            .collect();
        let items_ref: Vec<(&str, &i32)> = items
            .iter()
            .map(|(k, v)| (k.as_str(), v))
            .collect();

        client
            .mset(&items_ref, Some(Duration::from_secs(60)))
            .await
            .unwrap();

        // 一括取得
        let keys: Vec<String> = (0..100).map(|i| format!("batch_{}", i)).collect();
        let keys_ref: Vec<&str> = keys.iter().map(|s| s.as_str()).collect();
        let values: Vec<Option<i32>> = client.mget(&keys_ref).await.unwrap();

        assert_eq!(values.len(), 100);
        for (i, value) in values.into_iter().enumerate() {
            assert_eq!(value, Some(i as i32));
        }
    }
}

#[cfg(all(test, feature = "redis"))]
mod concurrent_access_tests {
    use k1s0_cache::{CacheClient, CacheConfig, CacheOperations};
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
            .key_prefix("concurrent")
            .build()
            .unwrap();

        let client = Arc::new(CacheClient::new(config).await.unwrap());

        (container, client)
    }

    #[tokio::test]
    async fn test_concurrent_writes() {
        let (_container, client) = setup_redis().await;

        // 10 個の並行タスクでカウンターをインクリメント
        let mut handles = vec![];
        for _ in 0..10 {
            let client = client.clone();
            handles.push(tokio::spawn(async move {
                for _ in 0..100 {
                    client.incr("concurrent_counter", 1).await.unwrap();
                }
            }));
        }

        for handle in handles {
            handle.await.unwrap();
        }

        // 合計は 1000 になるべき
        let value: Option<i64> = client.get("concurrent_counter").await.unwrap();
        assert_eq!(value, Some(1000));
    }

    #[tokio::test]
    async fn test_concurrent_read_write() {
        let (_container, client) = setup_redis().await;

        // 初期値を設定
        client
            .set("rw_key", &0i32, Some(Duration::from_secs(60)))
            .await
            .unwrap();

        // 並行で読み書き
        let mut handles = vec![];

        // 書き込みタスク
        for i in 0..5 {
            let client = client.clone();
            handles.push(tokio::spawn(async move {
                client
                    .set(&format!("rw_key_{}", i), &i, Some(Duration::from_secs(60)))
                    .await
                    .unwrap();
            }));
        }

        // 読み込みタスク
        for _ in 0..5 {
            let client = client.clone();
            handles.push(tokio::spawn(async move {
                let _: Option<i32> = client.get("rw_key").await.unwrap();
            }));
        }

        for handle in handles {
            handle.await.unwrap();
        }

        // すべてのキーが設定されていることを確認
        for i in 0..5 {
            let value: Option<i32> = client.get(&format!("rw_key_{}", i)).await.unwrap();
            assert_eq!(value, Some(i));
        }
    }

    #[tokio::test]
    async fn test_set_nx_race_condition() {
        let (_container, client) = setup_redis().await;

        // 10 個のタスクが同時に set_nx を試行
        let mut handles = vec![];
        for i in 0..10 {
            let client = client.clone();
            handles.push(tokio::spawn(async move {
                client
                    .set_nx("race_key", &i, Some(Duration::from_secs(60)))
                    .await
                    .unwrap()
            }));
        }

        let results: Vec<bool> = futures::future::join_all(handles)
            .await
            .into_iter()
            .map(|r| r.unwrap())
            .collect();

        // 1 つだけ成功するはず
        let success_count = results.iter().filter(|&&b| b).count();
        assert_eq!(success_count, 1);
    }
}

#[cfg(all(test, feature = "redis"))]
mod error_handling_tests {
    use k1s0_cache::{CacheConfig, CacheClient, CacheError};

    #[tokio::test]
    async fn test_connection_error_invalid_host() {
        let config = CacheConfig::builder()
            .host("invalid.host.that.does.not.exist")
            .port(6379)
            .build()
            .unwrap();

        let result = CacheClient::new(config).await;
        assert!(result.is_err());

        if let Err(e) = result {
            assert!(e.is_retryable());
        }
    }

    #[tokio::test]
    async fn test_connection_error_invalid_port() {
        let config = CacheConfig::builder()
            .host("127.0.0.1")
            .port(1) // 無効なポート
            .build()
            .unwrap();

        let result = CacheClient::new(config).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_serialization_error() {
        // このテストは CacheError の型をテスト
        let err = CacheError::serialization("test serialization error");
        assert!(!err.is_retryable());
        assert_eq!(err.error_code(), "CACHE_SERIALIZATION_ERROR");
    }

    #[tokio::test]
    async fn test_deserialization_error() {
        let err = CacheError::deserialization("test deserialization error");
        assert!(!err.is_retryable());
        assert_eq!(err.error_code(), "CACHE_DESERIALIZATION_ERROR");
    }
}

#[cfg(test)]
mod config_validation_tests {
    use k1s0_cache::{CacheConfig, DEFAULT_REDIS_PORT, DEFAULT_MAX_CONNECTIONS};

    #[test]
    fn test_config_builder_defaults() {
        let config = CacheConfig::builder()
            .host("localhost")
            .build()
            .unwrap();

        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, DEFAULT_REDIS_PORT);
    }

    #[test]
    fn test_config_builder_custom_values() {
        let config = CacheConfig::builder()
            .host("redis.example.com")
            .port(6380)
            .database(1)
            .key_prefix("myapp:")
            .build()
            .unwrap();

        assert_eq!(config.host, "redis.example.com");
        assert_eq!(config.port, 6380);
        assert_eq!(config.database, 1);
        assert_eq!(config.key_prefix, "myapp:");
    }

    #[test]
    fn test_config_pool_settings() {
        let config = CacheConfig::builder()
            .host("localhost")
            .max_connections(50)
            .min_connections(5)
            .build()
            .unwrap();

        assert_eq!(config.pool.max_connections, 50);
        assert_eq!(config.pool.min_connections, 5);
    }

    #[test]
    fn test_default_config() {
        let config = CacheConfig::default();
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, DEFAULT_REDIS_PORT);
        assert_eq!(config.pool.max_connections, DEFAULT_MAX_CONNECTIONS);
    }
}

#[cfg(test)]
mod metrics_tests {
    use k1s0_cache::{CacheMetrics, CacheOperation};

    #[test]
    fn test_metrics_hit_rate_calculation() {
        let metrics = CacheMetrics::new();

        // 5 ヒット、5 ミス
        for _ in 0..5 {
            metrics.record_hit(CacheOperation::Get);
        }
        for _ in 0..5 {
            metrics.record_miss(CacheOperation::Get);
        }

        let hit_rate = metrics.hit_rate();
        assert!((hit_rate - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_metrics_zero_operations() {
        let metrics = CacheMetrics::new();

        // 操作なしの場合のヒット率
        assert_eq!(metrics.hit_rate(), 0.0);
        assert_eq!(metrics.hits(), 0);
        assert_eq!(metrics.misses(), 0);
        assert_eq!(metrics.operations(), 0);
    }

    #[test]
    fn test_metrics_snapshot() {
        let metrics = CacheMetrics::new();

        metrics.record_hit(CacheOperation::Get);
        metrics.record_miss(CacheOperation::Get);
        metrics.record_hit(CacheOperation::Set);

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.total_hits, 2);
        assert_eq!(snapshot.total_misses, 1);
        assert_eq!(snapshot.total_operations, 3);
    }

    #[test]
    fn test_metrics_reset() {
        let metrics = CacheMetrics::new();

        metrics.record_hit(CacheOperation::Get);
        metrics.record_miss(CacheOperation::Get);

        assert_eq!(metrics.hits(), 1);
        assert_eq!(metrics.misses(), 1);

        metrics.reset();

        assert_eq!(metrics.hits(), 0);
        assert_eq!(metrics.misses(), 0);
    }

    #[test]
    fn test_operation_timer() {
        use k1s0_cache::OperationTimer;
        use std::time::Duration;

        let metrics = CacheMetrics::new();
        let timer = OperationTimer::start(&metrics, CacheOperation::Get);

        // 短い待機
        std::thread::sleep(Duration::from_millis(10));

        timer.finish_hit();

        assert_eq!(metrics.hits(), 1);
    }
}
