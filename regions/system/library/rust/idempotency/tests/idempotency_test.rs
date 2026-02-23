use k1s0_idempotency::{
    IdempotencyRecord, IdempotencyStatus, InMemoryIdempotencyStore, IdempotencyStore,
};

#[tokio::test]
async fn test_insert_and_get() {
    let store = InMemoryIdempotencyStore::new();
    let record = IdempotencyRecord::new("key-1".to_string(), None);
    store.insert(record).await.unwrap();

    let fetched = store.get("key-1").await.unwrap();
    assert!(fetched.is_some());
    let fetched = fetched.unwrap();
    assert_eq!(fetched.key, "key-1");
    assert_eq!(fetched.status, IdempotencyStatus::Pending);
}

#[tokio::test]
async fn test_insert_duplicate_returns_error() {
    let store = InMemoryIdempotencyStore::new();
    let record1 = IdempotencyRecord::new("dup-key".to_string(), None);
    let record2 = IdempotencyRecord::new("dup-key".to_string(), None);

    store.insert(record1).await.unwrap();
    let result = store.insert(record2).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("dup-key"));
}

#[tokio::test]
async fn test_get_nonexistent_returns_none() {
    let store = InMemoryIdempotencyStore::new();
    let result = store.get("nonexistent").await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_update_to_completed() {
    let store = InMemoryIdempotencyStore::new();
    let record = IdempotencyRecord::new("complete-key".to_string(), None);
    store.insert(record).await.unwrap();

    store
        .update(
            "complete-key",
            IdempotencyStatus::Completed,
            Some(r#"{"ok":true}"#.to_string()),
            Some(200),
        )
        .await
        .unwrap();

    let fetched = store.get("complete-key").await.unwrap().unwrap();
    assert_eq!(fetched.status, IdempotencyStatus::Completed);
    assert_eq!(fetched.response_body.as_deref(), Some(r#"{"ok":true}"#));
    assert_eq!(fetched.response_status, Some(200));
    assert!(fetched.completed_at.is_some());
}

#[tokio::test]
async fn test_update_to_failed() {
    let store = InMemoryIdempotencyStore::new();
    let record = IdempotencyRecord::new("fail-key".to_string(), None);
    store.insert(record).await.unwrap();

    store
        .update(
            "fail-key",
            IdempotencyStatus::Failed,
            Some("internal error".to_string()),
            Some(500),
        )
        .await
        .unwrap();

    let fetched = store.get("fail-key").await.unwrap().unwrap();
    assert_eq!(fetched.status, IdempotencyStatus::Failed);
    assert_eq!(fetched.response_body.as_deref(), Some("internal error"));
    assert_eq!(fetched.response_status, Some(500));
    assert!(fetched.completed_at.is_some());
}

#[tokio::test]
async fn test_get_expired_record_returns_none() {
    let store = InMemoryIdempotencyStore::new();
    // TTL 1秒で作成
    let record = IdempotencyRecord::new("expire-key".to_string(), Some(1));
    store.insert(record).await.unwrap();

    // 挿入直後は取得可能
    let fetched = store.get("expire-key").await.unwrap();
    assert!(fetched.is_some());

    // 期限切れを待つ
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    // 期限切れ後は None
    let fetched = store.get("expire-key").await.unwrap();
    assert!(fetched.is_none());
}

#[tokio::test]
async fn test_delete_existing() {
    let store = InMemoryIdempotencyStore::new();
    let record = IdempotencyRecord::new("del-key".to_string(), None);
    store.insert(record).await.unwrap();

    let deleted = store.delete("del-key").await.unwrap();
    assert!(deleted);

    let fetched = store.get("del-key").await.unwrap();
    assert!(fetched.is_none());
}

#[tokio::test]
async fn test_delete_nonexistent_returns_false() {
    let store = InMemoryIdempotencyStore::new();
    let deleted = store.delete("no-such-key").await.unwrap();
    assert!(!deleted);
}

#[tokio::test]
async fn test_record_is_expired_true() {
    // expires_at を過去に設定
    let mut record = IdempotencyRecord::new("exp-test".to_string(), None);
    record.expires_at = Some(chrono::Utc::now() - chrono::Duration::seconds(10));
    assert!(record.is_expired());
}

#[tokio::test]
async fn test_record_complete_sets_fields() {
    let record = IdempotencyRecord::new("comp-test".to_string(), None);
    assert_eq!(record.status, IdempotencyStatus::Pending);
    assert!(record.completed_at.is_none());

    let completed = record.complete(Some("response data".to_string()), Some(201));
    assert_eq!(completed.status, IdempotencyStatus::Completed);
    assert_eq!(completed.response_body.as_deref(), Some("response data"));
    assert_eq!(completed.response_status, Some(201));
    assert!(completed.completed_at.is_some());
}
