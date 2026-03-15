use std::sync::Arc;
use std::time::Duration;

use k1s0_distributed_lock::{DistributedLock, InMemoryDistributedLock, LockError, LockGuard};

// ---------------------------------------------------------------------------
// acquire and release basic flow
// ---------------------------------------------------------------------------

// ロックを取得・解放する基本フローが正常に動作することを確認する。
#[tokio::test]
async fn test_acquire_and_release_basic_flow() {
    let lock = InMemoryDistributedLock::new();

    let guard = lock
        .acquire("resource-1", Duration::from_secs(10))
        .await
        .unwrap();
    assert_eq!(guard.key, "resource-1");
    assert!(!guard.token.is_empty());
    assert!(lock.is_locked("resource-1").await.unwrap());

    lock.release(guard).await.unwrap();
    assert!(!lock.is_locked("resource-1").await.unwrap());
}

// ロック取得のたびに一意のトークンが生成されることを確認する。
#[tokio::test]
async fn test_acquire_returns_unique_tokens() {
    let lock = InMemoryDistributedLock::new();

    let g1 = lock
        .acquire("key-a", Duration::from_secs(10))
        .await
        .unwrap();
    let g1_token = g1.token.clone();
    lock.release(g1).await.unwrap();

    let g2 = lock
        .acquire("key-a", Duration::from_secs(10))
        .await
        .unwrap();
    // Tokens should differ between separate acquisitions
    assert_ne!(g1_token, g2.token);

    lock.release(g2).await.unwrap();
}

// ---------------------------------------------------------------------------
// double acquire returns AlreadyLocked
// ---------------------------------------------------------------------------

// 既にロック済みのキーを再取得すると AlreadyLocked エラーが返ることを確認する。
#[tokio::test]
async fn test_double_acquire_returns_already_locked() {
    let lock = InMemoryDistributedLock::new();

    let _guard = lock
        .acquire("key-1", Duration::from_secs(10))
        .await
        .unwrap();
    let result = lock.acquire("key-1", Duration::from_secs(10)).await;

    assert!(result.is_err());
    match result {
        Err(LockError::AlreadyLocked(key)) => assert_eq!(key, "key-1"),
        Err(other) => panic!("expected AlreadyLocked, got: {other}"),
        Ok(_) => panic!("expected error but got Ok"),
    }
}

// 異なるキーは互いに独立して取得できることを確認する。
#[tokio::test]
async fn test_different_keys_can_be_acquired_independently() {
    let lock = InMemoryDistributedLock::new();

    let g1 = lock
        .acquire("key-a", Duration::from_secs(10))
        .await
        .unwrap();
    let g2 = lock
        .acquire("key-b", Duration::from_secs(10))
        .await
        .unwrap();

    assert!(lock.is_locked("key-a").await.unwrap());
    assert!(lock.is_locked("key-b").await.unwrap());

    lock.release(g1).await.unwrap();
    lock.release(g2).await.unwrap();
}

// ---------------------------------------------------------------------------
// extend updates TTL
// ---------------------------------------------------------------------------

// TTL を延長するとロックが期限切れせずに保持されることを確認する。
#[tokio::test]
async fn test_extend_updates_ttl() {
    let lock = InMemoryDistributedLock::new();

    // Acquire with a very short TTL
    let guard = lock
        .acquire("key-1", Duration::from_millis(50))
        .await
        .unwrap();

    // Extend with a long TTL
    lock.extend(&guard, Duration::from_secs(60)).await.unwrap();

    // After the original TTL would have expired, lock should still be held
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert!(lock.is_locked("key-1").await.unwrap());

    lock.release(guard).await.unwrap();
}

// 誤ったトークンで延長すると TokenMismatch エラーが返ることを確認する。
#[tokio::test]
async fn test_extend_with_wrong_token_returns_token_mismatch() {
    let lock = InMemoryDistributedLock::new();

    let _guard = lock
        .acquire("key-1", Duration::from_secs(10))
        .await
        .unwrap();
    let fake_guard = LockGuard {
        key: "key-1".to_string(),
        token: "wrong-token".to_string(),
    };

    let result = lock.extend(&fake_guard, Duration::from_secs(30)).await;
    assert!(matches!(result, Err(LockError::TokenMismatch)));
}

// 存在しないロックを延長しようとすると LockNotFound エラーが返ることを確認する。
#[tokio::test]
async fn test_extend_nonexistent_lock_returns_not_found() {
    let lock = InMemoryDistributedLock::new();

    let fake_guard = LockGuard {
        key: "no-such-key".to_string(),
        token: "some-token".to_string(),
    };

    let result = lock.extend(&fake_guard, Duration::from_secs(10)).await;
    assert!(matches!(result, Err(LockError::LockNotFound(_))));
}

// ---------------------------------------------------------------------------
// acquire after expiry succeeds
// ---------------------------------------------------------------------------

// TTL 期限切れ後に同じキーを再取得できることを確認する。
#[tokio::test]
async fn test_acquire_after_expiry_succeeds() {
    let lock = InMemoryDistributedLock::new();

    let _guard = lock
        .acquire("key-1", Duration::from_millis(1))
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_millis(20)).await;

    // Lock should have expired
    assert!(!lock.is_locked("key-1").await.unwrap());

    // Re-acquire should succeed
    let guard2 = lock
        .acquire("key-1", Duration::from_secs(10))
        .await
        .unwrap();
    assert!(lock.is_locked("key-1").await.unwrap());

    lock.release(guard2).await.unwrap();
}

// TTL 期限切れ後に is_locked が false を返すことを確認する。
#[tokio::test]
async fn test_is_locked_returns_false_after_expiry() {
    let lock = InMemoryDistributedLock::new();

    let _guard = lock
        .acquire("key-1", Duration::from_millis(1))
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_millis(20)).await;

    assert!(!lock.is_locked("key-1").await.unwrap());
}

// ---------------------------------------------------------------------------
// release non-existent lock
// ---------------------------------------------------------------------------

// 未取得のロックをリリースすると LockNotFound エラーが返ることを確認する。
#[tokio::test]
async fn test_release_nonexistent_lock_returns_lock_not_found() {
    let lock = InMemoryDistributedLock::new();

    let guard = LockGuard {
        key: "never-acquired".to_string(),
        token: "some-token".to_string(),
    };

    let result = lock.release(guard).await;
    match result {
        Err(LockError::LockNotFound(key)) => assert_eq!(key, "never-acquired"),
        Err(other) => panic!("expected LockNotFound, got: {other}"),
        Ok(_) => panic!("expected error but got Ok"),
    }
}

// 誤ったトークンでリリースすると TokenMismatch エラーが返ることを確認する。
#[tokio::test]
async fn test_release_with_wrong_token_returns_token_mismatch() {
    let lock = InMemoryDistributedLock::new();

    let _guard = lock
        .acquire("key-1", Duration::from_secs(10))
        .await
        .unwrap();

    let fake_guard = LockGuard {
        key: "key-1".to_string(),
        token: "wrong-token".to_string(),
    };

    let result = lock.release(fake_guard).await;
    assert!(matches!(result, Err(LockError::TokenMismatch)));
}

// ---------------------------------------------------------------------------
// concurrent lock attempts
// ---------------------------------------------------------------------------

// 同時並行でロック取得を試みた場合に1つだけ成功することを確認する。
#[tokio::test]
async fn test_concurrent_acquire_only_one_succeeds() {
    let lock = Arc::new(InMemoryDistributedLock::new());
    let mut handles = Vec::new();

    for _ in 0..10 {
        let lock = Arc::clone(&lock);
        handles.push(tokio::spawn(async move {
            lock.acquire("contested-key", Duration::from_secs(10)).await
        }));
    }

    let results: Vec<_> = futures_collect(handles).await;

    let successes = results.iter().filter(|r| r.is_ok()).count();
    let failures = results.iter().filter(|r| r.is_err()).count();

    assert_eq!(
        successes, 1,
        "exactly one concurrent acquire should succeed"
    );
    assert_eq!(failures, 9, "remaining concurrent acquires should fail");

    // Verify all failures are AlreadyLocked
    for result in &results {
        if let Err(e) = result {
            assert!(
                matches!(e, LockError::AlreadyLocked(_)),
                "expected AlreadyLocked, got: {e}"
            );
        }
    }
}

async fn futures_collect(
    handles: Vec<tokio::task::JoinHandle<Result<LockGuard, LockError>>>,
) -> Vec<Result<LockGuard, LockError>> {
    let mut results = Vec::new();
    for h in handles {
        results.push(h.await.unwrap());
    }
    results
}

// ロックの取得・解放サイクルを繰り返しても正常に動作することを確認する。
#[tokio::test]
async fn test_concurrent_acquire_and_release_cycle() {
    let lock = Arc::new(InMemoryDistributedLock::new());

    for _ in 0..20 {
        let guard = lock
            .acquire("cycle-key", Duration::from_secs(10))
            .await
            .unwrap();
        lock.release(guard).await.unwrap();
    }

    assert!(!lock.is_locked("cycle-key").await.unwrap());
}

// ---------------------------------------------------------------------------
// is_locked edge cases
// ---------------------------------------------------------------------------

// 一度も取得されていないキーに対して is_locked が false を返すことを確認する。
#[tokio::test]
async fn test_is_locked_returns_false_for_never_acquired() {
    let lock = InMemoryDistributedLock::new();
    assert!(!lock.is_locked("nonexistent").await.unwrap());
}

// ロック解放後に is_locked が false を返すことを確認する。
#[tokio::test]
async fn test_is_locked_returns_false_after_release() {
    let lock = InMemoryDistributedLock::new();

    let guard = lock
        .acquire("key-1", Duration::from_secs(10))
        .await
        .unwrap();
    assert!(lock.is_locked("key-1").await.unwrap());

    lock.release(guard).await.unwrap();
    assert!(!lock.is_locked("key-1").await.unwrap());
}

// ---------------------------------------------------------------------------
// Default trait
// ---------------------------------------------------------------------------

// Default トレイトで生成したロックが正常に機能することを確認する。
#[tokio::test]
async fn test_default_trait_creates_usable_lock() {
    let lock = InMemoryDistributedLock::default();

    let guard = lock
        .acquire("key-1", Duration::from_secs(10))
        .await
        .unwrap();
    assert!(lock.is_locked("key-1").await.unwrap());
    lock.release(guard).await.unwrap();
}
