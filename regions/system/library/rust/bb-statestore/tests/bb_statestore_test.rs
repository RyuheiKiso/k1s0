#![allow(clippy::unwrap_used)]
// bb-statestore の外部結合テスト。
// InMemoryStateStore の get/set/delete（ETag、bulk 含む）を検証する。

use k1s0_bb_core::{Component, ComponentStatus};
use k1s0_bb_statestore::{InMemoryStateStore, StateStore, StateStoreError};

// --- ライフサイクルテスト ---

// InMemoryStateStore の init/close ライフサイクルが正しく動作することを確認する。
#[tokio::test]
async fn test_statestore_lifecycle() {
    let store = InMemoryStateStore::new("test-store");

    assert_eq!(store.status().await, ComponentStatus::Uninitialized);

    store.init().await.unwrap();
    assert_eq!(store.status().await, ComponentStatus::Ready);

    store.close().await.unwrap();
    assert_eq!(store.status().await, ComponentStatus::Closed);
}

// InMemoryStateStore のコンポーネントメタデータが正しいことを確認する。
#[test]
fn test_statestore_component_metadata() {
    let store = InMemoryStateStore::new("test-store");
    assert_eq!(store.name(), "test-store");
    assert_eq!(store.component_type(), "statestore");
    let meta = store.metadata();
    assert_eq!(meta.get("backend").unwrap(), "memory");
}

// --- get/set テスト ---

// 値をセットして正しく取得できることを確認する。
#[tokio::test]
async fn test_set_and_get() {
    let store = InMemoryStateStore::new("test-store");
    store.init().await.unwrap();

    let etag = store.set("user:1", b"Alice", None).await.unwrap();
    assert!(!etag.is_empty());

    let entry = store.get("user:1").await.unwrap().unwrap();
    assert_eq!(entry.key, "user:1");
    assert_eq!(entry.value, b"Alice");
    assert_eq!(entry.etag, etag);
}

// 存在しないキーを取得すると None が返されることを確認する。
#[tokio::test]
async fn test_get_not_found() {
    let store = InMemoryStateStore::new("test-store");
    let result = store.get("missing").await.unwrap();
    assert!(result.is_none());
}

// 値を上書きした場合に新しい ETag が返され、古い ETag と異なることを確認する。
#[tokio::test]
async fn test_set_overwrites_value() {
    let store = InMemoryStateStore::new("test-store");

    let etag1 = store.set("key", b"value1", None).await.unwrap();
    let etag2 = store.set("key", b"value2", None).await.unwrap();

    assert_ne!(etag1, etag2);

    let entry = store.get("key").await.unwrap().unwrap();
    assert_eq!(entry.value, b"value2");
    assert_eq!(entry.etag, etag2);
}

// --- ETag テスト ---

// 正しい ETag を指定して値を更新できることを確認する。
#[tokio::test]
async fn test_set_with_correct_etag() {
    let store = InMemoryStateStore::new("test-store");

    let etag = store.set("key", b"v1", None).await.unwrap();
    let new_etag = store.set("key", b"v2", Some(&etag)).await.unwrap();
    assert_ne!(etag, new_etag);

    let entry = store.get("key").await.unwrap().unwrap();
    assert_eq!(entry.value, b"v2");
}

// 不正な ETag でセットすると ETagMismatch エラーになることを確認する。
#[tokio::test]
async fn test_set_with_wrong_etag() {
    let store = InMemoryStateStore::new("test-store");

    store.set("key", b"v1", None).await.unwrap();
    let result = store.set("key", b"v2", Some("wrong-etag")).await;
    assert!(matches!(result, Err(StateStoreError::ETagMismatch { .. })));
}

// --- delete テスト ---

// キーを削除後に取得すると None が返されることを確認する。
#[tokio::test]
async fn test_delete() {
    let store = InMemoryStateStore::new("test-store");

    store.set("key", b"value", None).await.unwrap();
    store.delete("key", None).await.unwrap();

    assert!(store.get("key").await.unwrap().is_none());
}

// 正しい ETag を指定して削除できることを確認する。
#[tokio::test]
async fn test_delete_with_correct_etag() {
    let store = InMemoryStateStore::new("test-store");

    let etag = store.set("key", b"value", None).await.unwrap();
    store.delete("key", Some(&etag)).await.unwrap();

    assert!(store.get("key").await.unwrap().is_none());
}

// 不正な ETag で削除すると ETagMismatch エラーになることを確認する。
#[tokio::test]
async fn test_delete_with_wrong_etag() {
    let store = InMemoryStateStore::new("test-store");

    store.set("key", b"value", None).await.unwrap();
    let result = store.delete("key", Some("wrong-etag")).await;
    assert!(matches!(result, Err(StateStoreError::ETagMismatch { .. })));

    // 値が削除されていないことを確認する
    assert!(store.get("key").await.unwrap().is_some());
}

// 存在しないキーを削除してもエラーにならないことを確認する。
#[tokio::test]
async fn test_delete_nonexistent_key() {
    let store = InMemoryStateStore::new("test-store");
    let result = store.delete("missing", None).await;
    assert!(result.is_ok());
}

// --- bulk テスト ---

// 複数キーを一括取得し、存在するキーのエントリのみ返されることを確認する。
#[tokio::test]
async fn test_bulk_get() {
    let store = InMemoryStateStore::new("test-store");

    store.set("a", b"1", None).await.unwrap();
    store.set("b", b"2", None).await.unwrap();
    store.set("c", b"3", None).await.unwrap();

    let entries = store.bulk_get(&["a", "c", "missing"]).await.unwrap();
    assert_eq!(entries.len(), 2);

    let keys: Vec<&str> = entries.iter().map(|e| e.key.as_str()).collect();
    assert!(keys.contains(&"a"));
    assert!(keys.contains(&"c"));
}

// 複数エントリを一括セットし、それぞれ ETag が返されることを確認する。
#[tokio::test]
async fn test_bulk_set() {
    let store = InMemoryStateStore::new("test-store");

    let etags = store
        .bulk_set(&[("x", b"10"), ("y", b"20"), ("z", b"30")])
        .await
        .unwrap();
    assert_eq!(etags.len(), 3);

    // 各エントリが正しく設定されていることを確認する
    let x = store.get("x").await.unwrap().unwrap();
    assert_eq!(x.value, b"10");

    let y = store.get("y").await.unwrap().unwrap();
    assert_eq!(y.value, b"20");

    let z = store.get("z").await.unwrap().unwrap();
    assert_eq!(z.value, b"30");
}

// 空のキーリストで bulk_get すると空の結果が返ることを確認する。
#[tokio::test]
async fn test_bulk_get_empty() {
    let store = InMemoryStateStore::new("test-store");
    let entries = store.bulk_get(&[]).await.unwrap();
    assert!(entries.is_empty());
}

// 空のエントリリストで bulk_set すると空の ETag リストが返ることを確認する。
#[tokio::test]
async fn test_bulk_set_empty() {
    let store = InMemoryStateStore::new("test-store");
    let etags = store.bulk_set(&[]).await.unwrap();
    assert!(etags.is_empty());
}

// --- close テスト ---

// close 後にストアがクリアされ、値が取得できなくなることを確認する。
#[tokio::test]
async fn test_close_clears_store() {
    let store = InMemoryStateStore::new("test-store");

    store.set("key", b"value", None).await.unwrap();
    store.close().await.unwrap();

    assert_eq!(store.status().await, ComponentStatus::Closed);
    assert!(store.get("key").await.unwrap().is_none());
}

// --- エラーテスト ---

// StateStoreError の各バリアントが正しいメッセージを持つことを確認する。
#[test]
fn test_state_store_error_display() {
    let err = StateStoreError::NotFound("key-123".to_string());
    assert!(err.to_string().contains("key-123"));

    let err = StateStoreError::ETagMismatch {
        expected: "etag-a".to_string(),
        actual: "etag-b".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("etag-a"));
    assert!(msg.contains("etag-b"));

    let err = StateStoreError::Connection("redis down".to_string());
    assert!(err.to_string().contains("redis down"));
}
