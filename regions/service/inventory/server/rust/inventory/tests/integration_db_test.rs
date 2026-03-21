// 在庫サーバーの実 DB 統合テスト（T-01/T-02 対応）
// このテストは実際の PostgreSQL に接続し、DB レイヤーの品質を保証する。
// 通常の `cargo test` では #[ignore] によりスキップされる。
// CI 環境（DATABASE_URL が設定済み）では `cargo test -- --include-ignored` で実行する。

use std::sync::Arc;

use k1s0_inventory_server::domain::repository::inventory_repository::InventoryRepository;
use k1s0_inventory_server::infrastructure::database::inventory_repository::InventoryPostgresRepository;
use k1s0_inventory_server::MIGRATOR;

/// テスト用 DB プールを構築するヘルパー関数。
/// DATABASE_URL 環境変数が設定されていない場合はパニックする。
async fn setup_test_db() -> sqlx::PgPool {
    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL が設定されていません（実 DB テストには必須）");
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("テスト用 DB への接続に失敗");

    // マイグレーション適用（べき等: 既に適用済みの場合はスキップ）
    MIGRATOR.run(&pool).await.expect("マイグレーションの適用に失敗");
    pool
}

/// テスト後に作成したデータを削除するクリーンアップヘルパー。
async fn cleanup(pool: &sqlx::PgPool, product_id: &str) {
    // outbox イベントから削除（外部キー制約のため）
    let _ = sqlx::query("DELETE FROM inventory_outbox WHERE aggregate_id IN (SELECT id::text FROM inventory_items WHERE product_id = $1)")
        .bind(product_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM inventory_items WHERE product_id = $1")
        .bind(product_id)
        .execute(pool)
        .await;
}

/// 在庫アイテムの基本 CRUD が実 DB 上で正しく動作することを検証する。
#[tokio::test]
#[ignore = "実 DB テスト: DATABASE_URL 環境変数が必要。CI では --include-ignored で実行する"]
async fn test_inventory_crud_with_real_db() {
    let pool = setup_test_db().await;
    let repo = InventoryPostgresRepository::new(pool.clone());
    let product_id = "test-product-db-crud";
    let warehouse_id = "test-warehouse-01";

    cleanup(&pool, product_id).await;

    // CREATE: 在庫アイテムを作成する
    let item = repo
        .create(product_id, warehouse_id, 100)
        .await
        .expect("在庫アイテムの作成に失敗");
    assert_eq!(item.product_id, product_id);
    assert_eq!(item.qty_available, 100);
    assert_eq!(item.qty_reserved, 0);
    assert_eq!(item.version, 1);

    // READ: ID で取得できる
    let found = repo
        .find_by_id(item.id)
        .await
        .expect("find_by_id に失敗")
        .expect("アイテムが見つからない");
    assert_eq!(found.id, item.id);
    assert_eq!(found.qty_available, 100);

    cleanup(&pool, product_id).await;
}

/// 楽観ロック（version チェック）が実 DB 上で正しく動作することを検証する。
/// 在庫予約時に version が一致しない場合はエラーになることを確認する（R-02 対応）。
#[tokio::test]
#[ignore = "実 DB テスト: DATABASE_URL 環境変数が必要。CI では --include-ignored で実行する"]
async fn test_optimistic_lock_with_real_db() {
    let pool = setup_test_db().await;
    let repo = Arc::new(InventoryPostgresRepository::new(pool.clone()));
    let product_id = "test-product-db-optlock";
    let warehouse_id = "test-warehouse-01";
    let order_id = "order-optlock-test-001";

    cleanup(&pool, product_id).await;

    // 初期在庫を作成する（qty_available=50, version=1）
    let item = repo
        .create(product_id, warehouse_id, 50)
        .await
        .expect("在庫アイテムの作成に失敗");
    assert_eq!(item.version, 1);

    // 正常な予約: version=1 で予約する
    let reserved = repo
        .reserve_stock(item.id, 10, item.version, order_id)
        .await
        .expect("在庫予約に失敗");
    assert_eq!(reserved.qty_available, 40);
    assert_eq!(reserved.qty_reserved, 10);
    assert_eq!(reserved.version, 2); // version がインクリメントされる

    // 楽観ロック違反: 古い version=1 で再度予約を試みるとエラーになる
    let stale_result = repo.reserve_stock(item.id, 5, 1, "order-stale").await;
    assert!(
        stale_result.is_err(),
        "古い version で予約を試みたときにエラーが発生することを期待する"
    );

    cleanup(&pool, product_id).await;
}

/// Outbox イベントの挿入とフェッチが実 DB 上で正しく動作することを検証する。
#[tokio::test]
#[ignore = "実 DB テスト: DATABASE_URL 環境変数が必要。CI では --include-ignored で実行する"]
async fn test_outbox_events_with_real_db() {
    let pool = setup_test_db().await;
    let repo = InventoryPostgresRepository::new(pool.clone());
    let product_id = "test-product-db-outbox";
    let warehouse_id = "test-warehouse-01";

    cleanup(&pool, product_id).await;

    // 在庫アイテムを作成する
    let item = repo
        .create(product_id, warehouse_id, 30)
        .await
        .expect("在庫アイテムの作成に失敗");

    // Outbox イベントを挿入する
    let payload = serde_json::json!({
        "inventory_item_id": item.id.to_string(),
        "quantity": 10
    });
    repo.insert_outbox_event("InventoryItem", &item.id.to_string(), "StockReserved", &payload)
        .await
        .expect("Outbox イベントの挿入に失敗");

    // 未パブリッシュのイベントを取得できる
    let events = repo
        .fetch_unpublished_events(10)
        .await
        .expect("fetch_unpublished_events に失敗");
    assert!(!events.is_empty(), "Outbox イベントが取得できること");

    // イベントをパブリッシュ済みとしてマークする
    let event_ids: Vec<_> = events.iter().map(|e| e.id).collect();
    repo.mark_events_published(&event_ids)
        .await
        .expect("mark_events_published に失敗");

    // マーク後は未パブリッシュのイベントが0件になる
    let remaining = repo
        .fetch_unpublished_events(10)
        .await
        .expect("fetch_unpublished_events に失敗");
    assert!(remaining.is_empty(), "パブリッシュ済みイベントは取得されないこと");

    cleanup(&pool, product_id).await;
}
