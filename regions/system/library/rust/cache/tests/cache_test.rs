use std::time::Duration;

use k1s0_cache::{CacheClient, InMemoryCacheClient};

#[tokio::test]
async fn test_set_and_get() {
    let cache = InMemoryCacheClient::new();
    cache.set("key1", "value1", None).await.unwrap();
    let result = cache.get("key1").await.unwrap();
    assert_eq!(result, Some("value1".to_string()));
}

#[tokio::test]
async fn test_get_nonexistent_returns_none() {
    let cache = InMemoryCacheClient::new();
    let result = cache.get("nonexistent").await.unwrap();
    assert_eq!(result, None);
}

#[tokio::test]
async fn test_delete_existing_key() {
    let cache = InMemoryCacheClient::new();
    cache.set("key1", "value1", None).await.unwrap();
    let deleted = cache.delete("key1").await.unwrap();
    assert!(deleted);
    let result = cache.get("key1").await.unwrap();
    assert_eq!(result, None);
}

#[tokio::test]
async fn test_delete_nonexistent_returns_false() {
    let cache = InMemoryCacheClient::new();
    let deleted = cache.delete("nonexistent").await.unwrap();
    assert!(!deleted);
}

#[tokio::test]
async fn test_exists_true_and_false() {
    let cache = InMemoryCacheClient::new();
    cache.set("key1", "value1", None).await.unwrap();
    assert!(cache.exists("key1").await.unwrap());
    assert!(!cache.exists("nonexistent").await.unwrap());
}

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

#[tokio::test]
async fn test_overwrite_existing_key() {
    let cache = InMemoryCacheClient::new();
    cache.set("key1", "value1", None).await.unwrap();
    cache.set("key1", "value2", None).await.unwrap();
    let result = cache.get("key1").await.unwrap();
    assert_eq!(result, Some("value2".to_string()));
}
