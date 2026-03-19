#![allow(clippy::unwrap_used)]
use k1s0_bulkhead::{Bulkhead, BulkheadConfig, BulkheadError};
use std::sync::Arc;
use std::time::Duration;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn test_config(max_concurrent: usize, wait_ms: u64) -> BulkheadConfig {
    BulkheadConfig {
        max_concurrent_calls: max_concurrent,
        max_wait_duration: Duration::from_millis(wait_ms),
    }
}

fn fast_config(max_concurrent: usize) -> BulkheadConfig {
    test_config(max_concurrent, 50)
}

// ===========================================================================
// Concurrent execution limit
// ===========================================================================

// 上限以内のパーミット取得がすべて成功することを確認する。
#[tokio::test]
async fn acquire_within_limit_succeeds() {
    let bh = Bulkhead::new(fast_config(3));

    let _p1 = bh.acquire().await.unwrap();
    let _p2 = bh.acquire().await.unwrap();
    let _p3 = bh.acquire().await.unwrap();

    let m = bh.metrics();
    assert_eq!(m.current_concurrent, 3);
}

// 1件のパーミット取得でメトリクスが正しく更新されることを確認する。
#[tokio::test]
async fn single_permit_acquire_and_release() {
    let bh = Bulkhead::new(fast_config(1));
    let permit = bh.acquire().await.unwrap();

    assert_eq!(bh.metrics().current_concurrent, 1);

    drop(permit);
    // Note: dropping the permit releases the semaphore slot, but record_release
    // is only called via call(). So current_concurrent in metrics stays at 1
    // after raw acquire/drop. This is by design.
}

// ===========================================================================
// Rejection when at capacity
// ===========================================================================

// 上限到達時のパーミット取得が Full エラーになることを確認する。
#[tokio::test]
async fn rejected_when_at_capacity() {
    let bh = Bulkhead::new(fast_config(1));
    let _permit = bh.acquire().await.unwrap();

    let result = bh.acquire().await;
    assert!(result.is_err());

    match result.unwrap_err() {
        BulkheadError::Full { max_concurrent } => assert_eq!(max_concurrent, 1),
        other => panic!("expected Full error, got {:?}", other),
    }
}

// 複数回のリジェクトがメトリクスの rejection_count に正しく計上されることを確認する。
#[tokio::test]
async fn multiple_rejections_counted() {
    let bh = Bulkhead::new(fast_config(1));
    let _permit = bh.acquire().await.unwrap();

    for _ in 0..3 {
        let _ = bh.acquire().await;
    }

    assert_eq!(bh.metrics().rejection_count, 3);
}

// バルクヘッドが満杯のとき call() が Full エラーを返すことを確認する。
#[tokio::test]
async fn rejection_via_call_when_full() {
    let bh = Bulkhead::new(fast_config(1));
    let _permit = bh.acquire().await.unwrap();

    let result: Result<i32, BulkheadError<String>> =
        bh.call(|| async { Ok::<i32, String>(42) }).await;
    assert!(result.is_err());

    match result.unwrap_err() {
        BulkheadError::Full { max_concurrent } => assert_eq!(max_concurrent, 1),
        other => panic!("expected Full error, got {:?}", other),
    }
}

// ===========================================================================
// Release permits after completion
// ===========================================================================

// call() 正常完了後にパーミットが解放され再取得できることを確認する。
#[tokio::test]
async fn call_releases_permit_after_success() {
    let bh = Bulkhead::new(fast_config(1));

    let result: Result<i32, BulkheadError<String>> =
        bh.call(|| async { Ok::<i32, String>(42) }).await;
    assert_eq!(result.unwrap(), 42);

    // After call completes, metrics should show release
    assert_eq!(bh.metrics().current_concurrent, 0);

    // Should be able to acquire again
    let _p = bh.acquire().await.unwrap();
}

// call() 内部エラー後もパーミットが解放され再取得できることを確認する。
#[tokio::test]
async fn call_releases_permit_after_inner_error() {
    let bh = Bulkhead::new(fast_config(1));

    let result: Result<i32, BulkheadError<String>> = bh
        .call(|| async { Err::<i32, String>("boom".to_string()) })
        .await;
    assert!(result.is_err());

    // The permit is dropped when call returns (via _permit scope),
    // so a new acquire should succeed even though the call failed.
    // Note: record_release is called in the Ok path of call(), not on error.
    // But the semaphore permit is dropped, so the slot is available.
    let _p = bh.acquire().await.unwrap();
}

// 連続した call() 呼び出しでパーミットが再利用されることを確認する。
#[tokio::test]
async fn sequential_calls_reuse_permits() {
    let bh = Bulkhead::new(fast_config(1));

    for i in 0..5 {
        let result: Result<i32, BulkheadError<String>> =
            bh.call(|| async move { Ok::<i32, String>(i) }).await;
        assert_eq!(result.unwrap(), i);
    }
}

// ===========================================================================
// Wait / timeout behavior
// ===========================================================================

// タイムアウト内にパーミットが解放されれば取得が成功することを確認する。
#[tokio::test]
async fn waits_for_permit_within_timeout() {
    let bh = Arc::new(Bulkhead::new(test_config(1, 500)));
    let permit = bh.acquire().await.unwrap();

    let bh_clone = bh.clone();
    let handle = tokio::spawn(async move {
        // This will wait for the permit to be released
        bh_clone.acquire().await
    });

    // Release after a short delay
    tokio::time::sleep(Duration::from_millis(50)).await;
    drop(permit);

    let result = handle.await.unwrap();
    assert!(result.is_ok());
}

// タイムアウト内にパーミットが解放されない場合に取得が失敗することを確認する。
#[tokio::test]
async fn times_out_waiting_for_permit() {
    let bh = Arc::new(Bulkhead::new(test_config(1, 50)));
    let _permit = bh.acquire().await.unwrap();

    let bh_clone = bh.clone();
    let handle = tokio::spawn(async move { bh_clone.acquire().await });

    let result = handle.await.unwrap();
    assert!(result.is_err());
}

// ===========================================================================
// call() tests
// ===========================================================================

// call() が成功時に正しい戻り値を返すことを確認する。
#[tokio::test]
async fn call_success_returns_value() {
    let bh = Bulkhead::new(fast_config(2));
    let result: Result<&str, BulkheadError<String>> =
        bh.call(|| async { Ok::<&str, String>("hello") }).await;
    assert_eq!(result.unwrap(), "hello");
}

// call() が内部エラーを Inner エラーとしてラップして返すことを確認する。
#[tokio::test]
async fn call_propagates_inner_error() {
    let bh = Bulkhead::new(fast_config(2));
    let result: Result<i32, BulkheadError<String>> = bh
        .call(|| async { Err::<i32, String>("inner fail".to_string()) })
        .await;

    match result {
        Err(BulkheadError::Inner(e)) => assert_eq!(e, "inner fail"),
        other => panic!("expected Inner error, got {:?}", other),
    }
}

// ===========================================================================
// Metrics tracking
// ===========================================================================

// 初期状態でメトリクスの各カウンタがゼロであることを確認する。
#[tokio::test]
async fn metrics_initial_state() {
    let bh = Bulkhead::new(fast_config(5));
    let m = bh.metrics();
    assert_eq!(m.current_concurrent, 0);
    assert_eq!(m.rejection_count, 0);
}

// パーミット取得のたびに current_concurrent が増加することを確認する。
#[tokio::test]
async fn metrics_track_acquire() {
    let bh = Bulkhead::new(fast_config(3));

    let _p1 = bh.acquire().await.unwrap();
    assert_eq!(bh.metrics().current_concurrent, 1);

    let _p2 = bh.acquire().await.unwrap();
    assert_eq!(bh.metrics().current_concurrent, 2);
}

// リジェクト発生のたびに rejection_count が増加することを確認する。
#[tokio::test]
async fn metrics_track_rejections() {
    let bh = Bulkhead::new(fast_config(1));
    let _permit = bh.acquire().await.unwrap();

    let _ = bh.acquire().await;
    let _ = bh.acquire().await;

    assert_eq!(bh.metrics().rejection_count, 2);
}

// call() 完了後に current_concurrent がゼロに戻ることを確認する。
#[tokio::test]
async fn metrics_track_call_lifecycle() {
    let bh = Bulkhead::new(fast_config(2));

    // Before call
    assert_eq!(bh.metrics().current_concurrent, 0);

    // After successful call, current_concurrent should be back to 0
    let _: Result<i32, BulkheadError<String>> = bh.call(|| async { Ok::<i32, String>(1) }).await;
    assert_eq!(bh.metrics().current_concurrent, 0);
}

// ===========================================================================
// Config validation / defaults
// ===========================================================================

// デフォルト設定の max_concurrent_calls と max_wait_duration が適切な値であることを確認する。
#[test]
fn default_config_has_sane_values() {
    let config = BulkheadConfig::default();
    assert_eq!(config.max_concurrent_calls, 20);
    assert_eq!(config.max_wait_duration, Duration::from_millis(500));
}

// 設定をクローンすると同一の値を持つことを確認する。
#[test]
fn config_clone_is_equal() {
    let config = BulkheadConfig {
        max_concurrent_calls: 10,
        max_wait_duration: Duration::from_secs(5),
    };
    let cloned = config.clone();
    assert_eq!(cloned.max_concurrent_calls, 10);
    assert_eq!(cloned.max_wait_duration, Duration::from_secs(5));
}

// ===========================================================================
// Concurrent access safety
// ===========================================================================

// 並行取得が上限に達した後の追加取得が失敗することを確認する。
#[tokio::test]
async fn concurrent_acquires_respect_limit() {
    let bh = Arc::new(Bulkhead::new(fast_config(5)));
    let mut handles = Vec::new();

    for _ in 0..5 {
        let bh_clone = bh.clone();
        handles.push(tokio::spawn(async move { bh_clone.acquire().await }));
    }

    let mut permits = Vec::new();
    for h in handles {
        permits.push(h.await.unwrap().unwrap());
    }

    assert_eq!(bh.metrics().current_concurrent, 5);

    // 6th should fail
    let result = bh.acquire().await;
    assert!(result.is_err());
}

// 上限内の並行 call() がすべて成功することを確認する。
#[tokio::test]
async fn concurrent_calls_within_limit() {
    let bh = Arc::new(Bulkhead::new(fast_config(10)));
    let mut handles = Vec::new();

    for i in 0..10 {
        let bh_clone = bh.clone();
        handles.push(tokio::spawn(async move {
            let r: Result<i32, BulkheadError<String>> =
                bh_clone.call(|| async move { Ok::<i32, String>(i) }).await;
            r
        }));
    }

    for h in handles {
        assert!(h.await.unwrap().is_ok());
    }
}

// ===========================================================================
// Error display
// ===========================================================================

// Full エラーの Display 出力が期待するメッセージを含むことを確認する。
#[test]
fn error_display_full() {
    let err: BulkheadError = BulkheadError::Full { max_concurrent: 5 };
    assert_eq!(
        format!("{}", err),
        "bulkhead full: max concurrent calls (5) reached"
    );
}

// Inner エラーの Display 出力が内部エラーメッセージを正しく表示することを確認する。
#[test]
fn error_display_inner() {
    let err: BulkheadError<String> = BulkheadError::Inner("service down".to_string());
    assert_eq!(format!("{}", err), "service down");
}

// エラーが Debug トレイトを実装し "Full" を含む文字列を出力することを確認する。
#[test]
fn error_is_debug() {
    let err: BulkheadError = BulkheadError::Full { max_concurrent: 3 };
    let debug = format!("{:?}", err);
    assert!(debug.contains("Full"));
}
