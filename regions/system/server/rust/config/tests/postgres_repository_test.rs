use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use k1s0_config_server::adapter::repository::config_postgres::ConfigPostgresRepository;
use k1s0_config_server::domain::entity::config_change_log::{
    ConfigChangeLog, CreateChangeLogRequest,
};
use k1s0_config_server::domain::entity::config_entry::ConfigEntry;
use k1s0_config_server::domain::repository::ConfigRepository;

/// テスト用の ConfigEntry を作成するヘルパー。
fn make_entry(namespace: &str, key: &str, value: serde_json::Value) -> ConfigEntry {
    ConfigEntry {
        id: Uuid::new_v4(),
        namespace: namespace.to_string(),
        key: key.to_string(),
        value_json: value,
        version: 1,
        description: Some(format!("{}/{}", namespace, key)),
        created_by: "test@example.com".to_string(),
        updated_by: "test@example.com".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

/// テスト用プールの search_path を config スキーマに設定する。
/// マイグレーションは config スキーマにテーブルを作成するが、
/// sqlx::test のデフォルト search_path は public のみのため。
async fn setup_search_path(pool: &PgPool) {
    sqlx::query("SET search_path TO config, public")
        .execute(pool)
        .await
        .unwrap();
}

/// search_path 設定済みの ConfigPostgresRepository を作成するヘルパー。
async fn make_repo(pool: PgPool) -> ConfigPostgresRepository {
    setup_search_path(&pool).await;
    ConfigPostgresRepository::new(pool)
}

// --- マイグレーター定義 ---
// sqlx::test マクロ用。マイグレーションを自動適用する。
static MIGRATOR: sqlx::migrate::Migrator =
    sqlx::migrate!("../../../database/config-db/migrations");

// --- CRUD テスト ---

#[sqlx::test(migrator = "MIGRATOR")]
async fn test_create_and_get_config_entry(pool: PgPool) {
    let repo = make_repo(pool).await;
    let entry = make_entry("test.namespace", "test_key", serde_json::json!(42));

    // Create
    let created = repo.create(&entry).await.unwrap();
    assert_eq!(created.namespace, "test.namespace");
    assert_eq!(created.key, "test_key");
    assert_eq!(created.value_json, serde_json::json!(42));
    assert_eq!(created.version, 1);

    // Get
    let found = repo
        .find_by_namespace_and_key("test.namespace", "test_key")
        .await
        .unwrap();
    assert!(found.is_some());
    let found = found.unwrap();
    assert_eq!(found.id, entry.id);
    assert_eq!(found.value_json, serde_json::json!(42));
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn test_get_nonexistent_returns_none(pool: PgPool) {
    let repo = make_repo(pool).await;

    let result = repo
        .find_by_namespace_and_key("nonexistent", "missing")
        .await
        .unwrap();
    assert!(result.is_none());
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn test_find_by_id(pool: PgPool) {
    let repo = make_repo(pool).await;
    let entry = make_entry("test.ns", "by_id_key", serde_json::json!("value"));

    let created = repo.create(&entry).await.unwrap();

    let found = repo.find_by_id(&created.id).await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().key, "by_id_key");

    // 存在しない ID
    let not_found = repo.find_by_id(&Uuid::new_v4()).await.unwrap();
    assert!(not_found.is_none());
}

// --- List テスト ---

#[sqlx::test(migrator = "MIGRATOR")]
async fn test_list_by_namespace(pool: PgPool) {
    let repo = make_repo(pool).await;

    // 3件作成
    for key in &["alpha", "beta", "gamma"] {
        let entry = make_entry("list.test", key, serde_json::json!(key));
        repo.create(&entry).await.unwrap();
    }

    let result = repo
        .list_by_namespace("list.test", 1, 10, None)
        .await
        .unwrap();
    assert_eq!(result.entries.len(), 3);
    assert_eq!(result.pagination.total_count, 3);
    assert!(!result.pagination.has_next);

    // ページネーション
    let page1 = repo
        .list_by_namespace("list.test", 1, 2, None)
        .await
        .unwrap();
    assert_eq!(page1.entries.len(), 2);
    assert!(page1.pagination.has_next);

    let page2 = repo
        .list_by_namespace("list.test", 2, 2, None)
        .await
        .unwrap();
    assert_eq!(page2.entries.len(), 1);
    assert!(!page2.pagination.has_next);
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn test_list_by_namespace_with_search(pool: PgPool) {
    let repo = make_repo(pool).await;

    for key in &["max_connections", "max_retries", "timeout"] {
        let entry = make_entry("search.test", key, serde_json::json!(100));
        repo.create(&entry).await.unwrap();
    }

    let result = repo
        .list_by_namespace("search.test", 1, 10, Some("max".to_string()))
        .await
        .unwrap();
    assert_eq!(result.entries.len(), 2);
    assert_eq!(result.pagination.total_count, 2);
}

// --- Update テスト（楽観的ロック） ---

#[sqlx::test(migrator = "MIGRATOR")]
async fn test_update_config_with_correct_version(pool: PgPool) {
    let repo = make_repo(pool).await;
    let entry = make_entry("update.test", "key1", serde_json::json!(10));
    repo.create(&entry).await.unwrap();

    let updated = repo
        .update(
            "update.test",
            "key1",
            &serde_json::json!(20),
            1, // expected_version
            Some("updated description".to_string()),
            "updater@example.com",
        )
        .await
        .unwrap();

    assert_eq!(updated.value_json, serde_json::json!(20));
    assert_eq!(updated.version, 2);
    assert_eq!(updated.updated_by, "updater@example.com");
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn test_update_version_conflict(pool: PgPool) {
    let repo = make_repo(pool).await;
    let entry = make_entry("conflict.test", "key1", serde_json::json!(10));
    repo.create(&entry).await.unwrap();

    // 正しいバージョンで更新（version 1 -> 2）
    repo.update(
        "conflict.test",
        "key1",
        &serde_json::json!(20),
        1,
        None,
        "user@example.com",
    )
    .await
    .unwrap();

    // 古いバージョンで更新を試みる -> version conflict
    let result = repo
        .update(
            "conflict.test",
            "key1",
            &serde_json::json!(30),
            1, // stale version (current is 2)
            None,
            "user@example.com",
        )
        .await;

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("version conflict"),
        "expected version conflict error, got: {}",
        err_msg
    );
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn test_update_nonexistent_returns_error(pool: PgPool) {
    let repo = make_repo(pool).await;

    let result = repo
        .update(
            "nonexistent",
            "missing",
            &serde_json::json!("value"),
            1,
            None,
            "user@example.com",
        )
        .await;

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("config not found"),
        "expected config not found error, got: {}",
        err_msg
    );
}

// --- Delete テスト ---

#[sqlx::test(migrator = "MIGRATOR")]
async fn test_delete_config_entry(pool: PgPool) {
    let repo = make_repo(pool).await;
    let entry = make_entry("delete.test", "key1", serde_json::json!("to_delete"));
    repo.create(&entry).await.unwrap();

    let deleted = repo.delete("delete.test", "key1").await.unwrap();
    assert!(deleted);

    // 削除後に取得すると None
    let found = repo
        .find_by_namespace_and_key("delete.test", "key1")
        .await
        .unwrap();
    assert!(found.is_none());
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn test_delete_nonexistent_returns_false(pool: PgPool) {
    let repo = make_repo(pool).await;

    let deleted = repo.delete("nonexistent", "missing").await.unwrap();
    assert!(!deleted);
}

// --- サービス設定テスト ---

#[sqlx::test(migrator = "MIGRATOR")]
async fn test_find_by_service_name_via_mappings(pool: PgPool) {
    let repo = make_repo(pool).await;

    // 設定エントリを作成（シードデータと衝突しないキーを使用）
    let entry1 = make_entry(
        "test.service.mapping",
        "max_connections",
        serde_json::json!(25),
    );
    let entry2 = make_entry(
        "test.service.mapping",
        "connection_timeout",
        serde_json::json!(30),
    );
    repo.create(&entry1).await.unwrap();
    repo.create(&entry2).await.unwrap();

    // service_config_mappings にマッピングを作成
    sqlx::query(
        "INSERT INTO service_config_mappings (service_name, config_entry_id) VALUES ($1, $2), ($1, $3)",
    )
    .bind("test-custom-service")
    .bind(entry1.id)
    .bind(entry2.id)
    .execute(repo.pool())
    .await
    .unwrap();

    let result = repo.find_by_service_name("test-custom-service").await.unwrap();
    assert_eq!(result.service_name, "test-custom-service");
    assert_eq!(result.entries.len(), 2);
}

// --- 変更ログテスト ---

#[sqlx::test(migrator = "MIGRATOR")]
async fn test_record_and_list_change_logs(pool: PgPool) {
    let repo = make_repo(pool).await;

    // 設定エントリを作成（変更ログの外部キー参照用）
    let entry = make_entry("log.test", "key1", serde_json::json!(10));
    repo.create(&entry).await.unwrap();

    // 変更ログを記録
    let log = ConfigChangeLog::new(CreateChangeLogRequest {
        config_entry_id: entry.id,
        namespace: "log.test".to_string(),
        key: "key1".to_string(),
        old_value: None,
        new_value: Some(serde_json::json!(10)),
        old_version: 0,
        new_version: 1,
        change_type: "CREATED".to_string(),
        changed_by: "test@example.com".to_string(),
    });
    repo.record_change_log(&log).await.unwrap();

    let log2 = ConfigChangeLog::new(CreateChangeLogRequest {
        config_entry_id: entry.id,
        namespace: "log.test".to_string(),
        key: "key1".to_string(),
        old_value: Some(serde_json::json!(10)),
        new_value: Some(serde_json::json!(20)),
        old_version: 1,
        new_version: 2,
        change_type: "UPDATED".to_string(),
        changed_by: "test@example.com".to_string(),
    });
    repo.record_change_log(&log2).await.unwrap();

    // 変更ログを取得
    let logs = repo.list_change_logs("log.test", "key1").await.unwrap();
    assert_eq!(logs.len(), 2);
    // DESC 順（新しい順）
    assert_eq!(logs[0].change_type, "UPDATED");
    assert_eq!(logs[1].change_type, "CREATED");
}
