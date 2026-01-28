//! Circuit Breaker の詳細テスト
//!
//! Half-Open 状態遷移、エッジケース、並行処理のテスト。

use k1s0_resilience::{
    CircuitBreaker, CircuitBreakerConfig, CircuitState, FailurePredicate, ResilienceError,
};
use std::sync::Arc;
use std::time::Duration;

/// Half-Open 状態のテスト
mod half_open_tests {
    use super::*;

    #[tokio::test]
    async fn test_transition_to_half_open() {
        let config = CircuitBreakerConfig::enabled()
            .failure_threshold(2)
            .reset_timeout_secs(1) // 1秒でリセット
            .build();
        let cb = CircuitBreaker::new(config);

        // Open にする
        cb.record_failure(&ResilienceError::timeout(1000));
        cb.record_failure(&ResilienceError::timeout(1000));
        assert_eq!(cb.state(), CircuitState::Open);

        // リクエスト拒否を確認
        assert!(!cb.allow_request());

        // 1秒待つ
        tokio::time::sleep(Duration::from_secs(1)).await;

        // Half-Open に遷移
        assert!(cb.allow_request());
        assert_eq!(cb.state(), CircuitState::HalfOpen);
    }

    #[tokio::test]
    async fn test_half_open_success_recovery() {
        let config = CircuitBreakerConfig::enabled()
            .failure_threshold(1)
            .success_threshold(2)
            .reset_timeout_secs(0) // 即座にリセット
            .build();
        let cb = CircuitBreaker::new(config);

        // Open にする
        cb.record_failure(&ResilienceError::timeout(1000));
        assert_eq!(cb.state(), CircuitState::Open);

        // Half-Open に遷移
        tokio::time::sleep(Duration::from_millis(10)).await;
        assert!(cb.allow_request());
        assert_eq!(cb.state(), CircuitState::HalfOpen);

        // 1回成功
        cb.record_success();
        assert_eq!(cb.state(), CircuitState::HalfOpen);

        // 2回目の成功 → Closed
        cb.record_success();
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_half_open_failure_returns_to_open() {
        let config = CircuitBreakerConfig::enabled()
            .failure_threshold(1)
            .success_threshold(3)
            .reset_timeout_secs(0)
            .build();
        let cb = CircuitBreaker::new(config);

        // Open にする
        cb.record_failure(&ResilienceError::timeout(1000));
        assert_eq!(cb.state(), CircuitState::Open);

        // Half-Open に遷移
        tokio::time::sleep(Duration::from_millis(10)).await;
        assert!(cb.allow_request());
        assert_eq!(cb.state(), CircuitState::HalfOpen);

        // 成功
        cb.record_success();
        assert_eq!(cb.state(), CircuitState::HalfOpen);

        // 失敗 → 即座に Open に戻る
        cb.record_failure(&ResilienceError::timeout(1000));
        assert_eq!(cb.state(), CircuitState::Open);
    }
}

/// 並行処理のテスト
mod concurrent_tests {
    use super::*;

    #[tokio::test]
    async fn test_concurrent_requests() {
        let config = CircuitBreakerConfig::enabled()
            .failure_threshold(5)
            .build();
        let cb = Arc::new(CircuitBreaker::new(config));

        // 10 個の並行タスク
        let mut handles = vec![];
        for _ in 0..10 {
            let cb = cb.clone();
            handles.push(tokio::spawn(async move {
                cb.execute(async { Ok::<_, ResilienceError>(42) }).await
            }));
        }

        let results: Vec<Result<i32, ResilienceError>> =
            futures::future::join_all(handles)
                .await
                .into_iter()
                .map(|r| r.unwrap())
                .collect();

        // すべて成功
        for result in results {
            assert!(result.is_ok());
        }

        // 失敗数は 0
        assert_eq!(cb.failure_count(), 0);
    }

    #[tokio::test]
    async fn test_concurrent_failures() {
        let config = CircuitBreakerConfig::enabled()
            .failure_threshold(5)
            .build();
        let cb = Arc::new(CircuitBreaker::new(config));

        // 10 個の並行タスクで失敗
        let mut handles = vec![];
        for _ in 0..10 {
            let cb = cb.clone();
            handles.push(tokio::spawn(async move {
                cb.execute(async { Err::<i32, _>(ResilienceError::timeout(1000)) })
                    .await
            }));
        }

        let _ = futures::future::join_all(handles).await;

        // Open になっているはず
        assert_eq!(cb.state(), CircuitState::Open);
    }
}

/// 失敗判定のテスト
mod failure_predicate_tests {
    use super::*;

    #[test]
    fn test_predicate_all() {
        let predicate = FailurePredicate::all();

        assert!(predicate.should_count(&ResilienceError::timeout(1000)));
        assert!(predicate.should_count(&ResilienceError::connection("refused")));
        assert!(predicate.should_count(&ResilienceError::internal("error")));

        // CircuitOpen と ConcurrencyLimit はカウントしない
        assert!(!predicate.should_count(&ResilienceError::circuit_open("open".to_string())));
        assert!(!predicate.should_count(&ResilienceError::concurrency_limit(10)));
    }

    #[test]
    fn test_predicate_timeout_only() {
        let predicate = FailurePredicate::timeout_only();

        assert!(predicate.should_count(&ResilienceError::timeout(1000)));
        assert!(!predicate.should_count(&ResilienceError::connection("refused")));
        assert!(!predicate.should_count(&ResilienceError::internal("error")));
    }

    #[tokio::test]
    async fn test_circuit_breaker_with_predicate() {
        let config = CircuitBreakerConfig::enabled()
            .failure_threshold(2)
            .failure_predicate(FailurePredicate::timeout_only())
            .build();
        let cb = CircuitBreaker::new(config);

        // 接続エラーは失敗としてカウントしない
        cb.record_failure(&ResilienceError::connection("refused"));
        cb.record_failure(&ResilienceError::connection("refused"));
        assert_eq!(cb.state(), CircuitState::Closed);

        // タイムアウトはカウント
        cb.record_failure(&ResilienceError::timeout(1000));
        cb.record_failure(&ResilienceError::timeout(1000));
        assert_eq!(cb.state(), CircuitState::Open);
    }
}

/// メトリクスのテスト
mod metrics_tests {
    use super::*;

    #[tokio::test]
    async fn test_rejected_count() {
        let config = CircuitBreakerConfig::enabled()
            .failure_threshold(1)
            .build();
        let cb = CircuitBreaker::new(config);

        // Open にする
        cb.record_failure(&ResilienceError::timeout(1000));

        // 3回試行（すべて拒否）
        for _ in 0..3 {
            let _ = cb.execute(async { Ok::<_, ResilienceError>(42) }).await;
        }

        assert_eq!(cb.metrics().rejected(), 3);
    }

    #[tokio::test]
    async fn test_state_transition_count() {
        let config = CircuitBreakerConfig::enabled()
            .failure_threshold(1)
            .success_threshold(1)
            .reset_timeout_secs(0)
            .build();
        let cb = CircuitBreaker::new(config);

        // Closed → Open
        cb.record_failure(&ResilienceError::timeout(1000));

        // Open → Half-Open
        tokio::time::sleep(Duration::from_millis(10)).await;
        assert!(cb.allow_request());

        // Half-Open → Closed
        cb.record_success();

        // 少なくとも 2 回の遷移（Closed→Open, Half-Open→Closed）
        assert!(cb.metrics().state_transitions() >= 2);
    }
}

/// エッジケースのテスト
mod edge_cases {
    use super::*;

    #[test]
    fn test_disabled_circuit_breaker() {
        let cb = CircuitBreaker::disabled();

        // 常に許可
        assert!(cb.allow_request());

        // 失敗を記録しても状態は変わらない
        for _ in 0..100 {
            cb.record_failure(&ResilienceError::timeout(1000));
        }
        assert!(cb.allow_request());
    }

    #[test]
    fn test_success_resets_failure_count() {
        let config = CircuitBreakerConfig::enabled()
            .failure_threshold(3)
            .build();
        let cb = CircuitBreaker::new(config);

        // 2回失敗
        cb.record_failure(&ResilienceError::timeout(1000));
        cb.record_failure(&ResilienceError::timeout(1000));
        assert_eq!(cb.failure_count(), 2);

        // 成功でリセット
        cb.record_success();
        assert_eq!(cb.failure_count(), 0);

        // 2回失敗しても Open にならない
        cb.record_failure(&ResilienceError::timeout(1000));
        cb.record_failure(&ResilienceError::timeout(1000));
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_execute_with_successful_recovery() {
        let config = CircuitBreakerConfig::enabled()
            .failure_threshold(2)
            .success_threshold(1)
            .reset_timeout_secs(0)
            .build();
        let cb = CircuitBreaker::new(config);

        // 失敗で Open
        let _ = cb.execute(async { Err::<i32, _>(ResilienceError::timeout(1000)) }).await;
        let _ = cb.execute(async { Err::<i32, _>(ResilienceError::timeout(1000)) }).await;
        assert_eq!(cb.state(), CircuitState::Open);

        // リセット待ち
        tokio::time::sleep(Duration::from_millis(10)).await;

        // 成功で復旧
        let result = cb.execute(async { Ok::<_, ResilienceError>(42) }).await;
        assert_eq!(result.unwrap(), 42);
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn test_config_builder_defaults() {
        let config = CircuitBreakerConfig::enabled().build();

        assert!(config.is_enabled());
        assert_eq!(config.failure_threshold(), 5);
        assert_eq!(config.success_threshold(), 3);
        assert_eq!(config.reset_timeout().as_secs(), 30);
    }

    #[test]
    fn test_config_serialization() {
        let config = CircuitBreakerConfig::enabled()
            .failure_threshold(10)
            .success_threshold(5)
            .build();

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: CircuitBreakerConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.failure_threshold(), 10);
        assert_eq!(deserialized.success_threshold(), 5);
        assert!(deserialized.is_enabled());
    }
}

/// Timeout のテスト
mod timeout_tests {
    use k1s0_resilience::{TimeoutConfig, TimeoutGuard};

    #[tokio::test]
    async fn test_timeout_success() {
        let guard = TimeoutGuard::new(TimeoutConfig::new(1000)).unwrap();

        let result = guard
            .execute(async {
                tokio::time::sleep(Duration::from_millis(10)).await;
                Ok::<_, ResilienceError>(42)
            })
            .await;

        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_timeout_exceeded() {
        let guard = TimeoutGuard::new(TimeoutConfig::new(100)).unwrap();

        let result = guard
            .execute(async {
                tokio::time::sleep(Duration::from_millis(500)).await;
                Ok::<_, ResilienceError>(42)
            })
            .await;

        assert!(matches!(result.unwrap_err(), ResilienceError::Timeout { .. }));
    }

    #[test]
    fn test_timeout_config_validation() {
        // 正常範囲
        assert!(TimeoutConfig::new(5000).validate().is_ok());

        // 下限未満（100ms 未満）
        assert!(TimeoutConfig::new(50).validate().is_err());

        // 上限超過（5分超）
        assert!(TimeoutConfig::new(400_000).validate().is_err());
    }
}

/// Concurrency Limiter のテスト
mod concurrency_tests {
    use k1s0_resilience::{ConcurrencyConfig, ConcurrencyLimiter};

    #[tokio::test]
    async fn test_concurrency_limit() {
        let limiter = ConcurrencyLimiter::new(ConcurrencyConfig::new(2));

        // 3つの並行タスクを開始
        let limiter = Arc::new(limiter);
        let mut handles = vec![];

        for i in 0..3 {
            let limiter = limiter.clone();
            handles.push(tokio::spawn(async move {
                limiter
                    .execute(async move {
                        tokio::time::sleep(Duration::from_millis(100)).await;
                        Ok::<_, ResilienceError>(i)
                    })
                    .await
            }));
        }

        let results: Vec<Result<i32, ResilienceError>> =
            futures::future::join_all(handles)
                .await
                .into_iter()
                .map(|r| r.unwrap())
                .collect();

        // 1つは制限に引っかかる可能性がある
        let success_count = results.iter().filter(|r| r.is_ok()).count();
        assert!(success_count >= 2);
    }
}

/// Bulkhead のテスト
mod bulkhead_tests {
    use k1s0_resilience::{Bulkhead, BulkheadConfig};

    #[tokio::test]
    async fn test_bulkhead_per_service() {
        let config = BulkheadConfig::new(100)
            .with_service_limit("auth-service", 2)
            .with_service_limit("data-service", 3);
        let bulkhead = Arc::new(Bulkhead::new(config));

        // auth-service に 3 つの並行リクエスト
        let mut handles = vec![];
        for i in 0..3 {
            let bulkhead = bulkhead.clone();
            handles.push(tokio::spawn(async move {
                bulkhead
                    .execute("auth-service", async move {
                        tokio::time::sleep(Duration::from_millis(100)).await;
                        Ok::<_, ResilienceError>(i)
                    })
                    .await
            }));
        }

        let results: Vec<Result<i32, ResilienceError>> =
            futures::future::join_all(handles)
                .await
                .into_iter()
                .map(|r| r.unwrap())
                .collect();

        // 2つは成功、1つは制限される可能性
        let success_count = results.iter().filter(|r| r.is_ok()).count();
        assert!(success_count >= 2);
    }

    #[tokio::test]
    async fn test_bulkhead_default_limit() {
        let config = BulkheadConfig::new(5);
        let bulkhead = Bulkhead::new(config);

        // 設定されていないサービスはデフォルト制限を使用
        let result = bulkhead
            .execute("unknown-service", async { Ok::<_, ResilienceError>(42) })
            .await;

        assert_eq!(result.unwrap(), 42);
    }
}
