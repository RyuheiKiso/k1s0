// bb-secretstore の外部結合テスト。
// InMemorySecretStore の get/set/delete（bulk 含む）を検証する。

use std::collections::HashMap;

use k1s0_bb_core::{Component, ComponentStatus};
use k1s0_bb_secretstore::{InMemorySecretStore, SecretStore, SecretStoreError};

// --- ライフサイクルテスト ---

// InMemorySecretStore の init/close ライフサイクルが正しく動作することを確認する。
#[tokio::test]
async fn test_secretstore_lifecycle() {
    let store = InMemorySecretStore::new("test-secrets");

    assert_eq!(store.status().await, ComponentStatus::Uninitialized);

    store.init().await.unwrap();
    assert_eq!(store.status().await, ComponentStatus::Ready);

    store.close().await.unwrap();
    assert_eq!(store.status().await, ComponentStatus::Closed);
}

// InMemorySecretStore のコンポーネントメタデータが正しいことを確認する。
#[test]
fn test_secretstore_component_metadata() {
    let store = InMemorySecretStore::new("test-secrets");
    assert_eq!(store.name(), "test-secrets");
    assert_eq!(store.component_type(), "secretstore");
    let meta = store.metadata();
    assert_eq!(meta.get("backend").unwrap(), "memory");
}

// --- get/set テスト ---

// シークレットを追加して正しく取得できることを確認する。
#[tokio::test]
async fn test_put_and_get_secret() {
    let store = InMemorySecretStore::new("test-secrets");
    store.put_secret("db/password", "s3cr3t").await;

    let secret = store.get_secret("db/password").await.unwrap();
    assert_eq!(secret.key, "db/password");
    assert_eq!(secret.value, "s3cr3t");
    assert!(secret.metadata.is_empty());
}

// 存在しないキーを取得しようとすると NotFound エラーになることを確認する。
#[tokio::test]
async fn test_get_secret_not_found() {
    let store = InMemorySecretStore::new("test-secrets");
    let result = store.get_secret("missing").await;
    assert!(matches!(result, Err(SecretStoreError::NotFound(_))));
}

// 同じキーに上書きすると新しい値が取得されることを確認する。
#[tokio::test]
async fn test_put_secret_overwrite() {
    let store = InMemorySecretStore::new("test-secrets");
    store.put_secret("db/password", "old-pass").await;
    store.put_secret("db/password", "new-pass").await;

    let secret = store.get_secret("db/password").await.unwrap();
    assert_eq!(secret.value, "new-pass");
}

// メタデータ付きシークレットを追加して正しく取得できることを確認する。
#[tokio::test]
async fn test_put_secret_with_metadata() {
    let store = InMemorySecretStore::new("test-secrets");
    let mut meta = HashMap::new();
    meta.insert("version".to_string(), "3".to_string());
    meta.insert("rotation".to_string(), "monthly".to_string());

    store
        .put_secret_with_metadata("api/key", "api-key-value", meta)
        .await;

    let secret = store.get_secret("api/key").await.unwrap();
    assert_eq!(secret.value, "api-key-value");
    assert_eq!(secret.metadata.get("version").unwrap(), "3");
    assert_eq!(secret.metadata.get("rotation").unwrap(), "monthly");
}

// --- bulk_get テスト ---

// 複数のシークレットを一括取得し、存在するキーのみ返されることを確認する。
#[tokio::test]
async fn test_bulk_get() {
    let store = InMemorySecretStore::new("test-secrets");
    store.put_secret("db/password", "pass1").await;
    store.put_secret("db/username", "admin").await;
    store.put_secret("api/key", "key123").await;

    let secrets = store
        .bulk_get(&["db/password", "api/key", "missing-key"])
        .await
        .unwrap();

    assert_eq!(secrets.len(), 2);
    assert_eq!(secrets.get("db/password").unwrap().value, "pass1");
    assert_eq!(secrets.get("api/key").unwrap().value, "key123");
    assert!(!secrets.contains_key("missing-key"));
}

// 空のキーリストで bulk_get すると空の結果が返ることを確認する。
#[tokio::test]
async fn test_bulk_get_empty_keys() {
    let store = InMemorySecretStore::new("test-secrets");
    store.put_secret("key1", "val1").await;

    let secrets = store.bulk_get(&[]).await.unwrap();
    assert!(secrets.is_empty());
}

// 全てのキーが存在しない場合に bulk_get が空の結果を返すことを確認する。
#[tokio::test]
async fn test_bulk_get_all_missing() {
    let store = InMemorySecretStore::new("test-secrets");
    let secrets = store.bulk_get(&["a", "b", "c"]).await.unwrap();
    assert!(secrets.is_empty());
}

// --- close テスト ---

// close 後にストアがクリアされシークレットが取得できなくなることを確認する。
#[tokio::test]
async fn test_close_clears_store() {
    let store = InMemorySecretStore::new("test-secrets");
    store.put_secret("db/password", "s3cr3t").await;

    store.close().await.unwrap();
    assert_eq!(store.status().await, ComponentStatus::Closed);

    // close 後はシークレットが取得できない（ストアがクリアされる）
    let result = store.get_secret("db/password").await;
    assert!(result.is_err());
}

// --- エラーテスト ---

// SecretStoreError の NotFound バリアントが正しいメッセージを持つことを確認する。
#[test]
fn test_secret_store_error_not_found() {
    let err = SecretStoreError::NotFound("missing-key".to_string());
    assert!(err.to_string().contains("missing-key"));
}

// SecretStoreError の PermissionDenied バリアントが正しいメッセージを持つことを確認する。
#[test]
fn test_secret_store_error_permission_denied() {
    let err = SecretStoreError::PermissionDenied("access denied".to_string());
    assert!(err.to_string().contains("access denied"));
}

// SecretStoreError の Connection バリアントが正しいメッセージを持つことを確認する。
#[test]
fn test_secret_store_error_connection() {
    let err = SecretStoreError::Connection("vault unreachable".to_string());
    assert!(err.to_string().contains("vault unreachable"));
}
