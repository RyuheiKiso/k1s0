use std::time::Duration;

use k1s0_cache::{CacheClient, InMemoryCacheClient};

// キャッシュにデータをセットして正常に取得できることを確認する。
#[tokio::test]
async fn test_set_and_get() {
    let cache = InMemoryCacheClient::new();
    cache.set("key1", "value1", None).await.unwrap();
    let result = cache.get("key1").await.unwrap();
    assert_eq!(result, Some("value1".to_string()));
}

// 存在しないキーを取得した場合に None が返ることを確認する。
#[tokio::test]
async fn test_get_nonexistent_returns_none() {
    let cache = InMemoryCacheClient::new();
    let result = cache.get("nonexistent").await.unwrap();
    assert_eq!(result, None);
}

// 存在するキーを削除してその後取得できないことを確認する。
#[tokio::test]
async fn test_delete_existing_key() {
    let cache = InMemoryCacheClient::new();
    cache.set("key1", "value1", None).await.unwrap();
    let deleted = cache.delete("key1").await.unwrap();
    assert!(deleted);
    let result = cache.get("key1").await.unwrap();
    assert_eq!(result, None);
}

// 存在しないキーを削除した場合に false が返ることを確認する。
#[tokio::test]
async fn test_delete_nonexistent_returns_false() {
    let cache = InMemoryCacheClient::new();
    let deleted = cache.delete("nonexistent").await.unwrap();
    assert!(!deleted);
}

// キーの存在確認が正しく true および false を返すことを検証する。
#[tokio::test]
async fn test_exists_true_and_false() {
    let cache = InMemoryCacheClient::new();
    cache.set("key1", "value1", None).await.unwrap();
    assert!(cache.exists("key1").await.unwrap());
    assert!(!cache.exists("nonexistent").await.unwrap());
}

// TTL 付きでセットしたデータが期限切れ後に取得できなくなることを確認する。
#[tokio::test]
async fn test_set_with_ttl_expires() {
    let cache = InMemoryCacheClient::new();
    cache
        .set("key1", "value1", Some(Duration::from_millis(50)))
        .await
        .unwrap();

    let result = cache.get("key1").await.unwrap();
    assert_eq!(result, Some("value1".to_string()));

    tokio::time::sleep(Duration::from_millis(100)).await;

    let result = cache.get("key1").await.unwrap();
    assert_eq!(result, None);
}

// キーが存在しない場合に set_nx が成功して値がセットされることを確認する。
#[tokio::test]
async fn test_set_nx_succeeds_when_not_exists() {
    let cache = InMemoryCacheClient::new();
    let result = cache
        .set_nx("key1", "value1", Duration::from_secs(10))
        .await
        .unwrap();
    assert!(result);
    let value = cache.get("key1").await.unwrap();
    assert_eq!(value, Some("value1".to_string()));
}

// キーが既に存在する場合に set_nx が失敗して元の値が保持されることを確認する。
#[tokio::test]
async fn test_set_nx_fails_when_exists() {
    let cache = InMemoryCacheClient::new();
    cache.set("key1", "value1", None).await.unwrap();
    let result = cache
        .set_nx("key1", "value2", Duration::from_secs(10))
        .await
        .unwrap();
    assert!(!result);
    let value = cache.get("key1").await.unwrap();
    assert_eq!(value, Some("value1".to_string()));
}

// expire でキーの TTL を更新し、期限切れ後に値が消えることを確認する。
#[tokio::test]
async fn test_expire_updates_ttl() {
    let cache = InMemoryCacheClient::new();
    cache.set("key1", "value1", None).await.unwrap();

    let updated = cache
        .expire("key1", Duration::from_millis(50))
        .await
        .unwrap();
    assert!(updated);

    let result = cache.get("key1").await.unwrap();
    assert_eq!(result, Some("value1".to_string()));

    tokio::time::sleep(Duration::from_millis(100)).await;

    let result = cache.get("key1").await.unwrap();
    assert_eq!(result, None);
}

// 既存のキーに新しい値をセットすると上書きされることを確認する。
#[tokio::test]
async fn test_overwrite_existing_key() {
    let cache = InMemoryCacheClient::new();
    cache.set("key1", "value1", None).await.unwrap();
    cache.set("key1", "value2", None).await.unwrap();
    let result = cache.get("key1").await.unwrap();
    assert_eq!(result, Some("value2".to_string()));
}
