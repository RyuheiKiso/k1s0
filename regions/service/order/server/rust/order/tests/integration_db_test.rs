// 注文サーバーの実 DB 統合テスト（T-01/T-02 対応）
// このテストは実際の PostgreSQL に接続し、DB レイヤーの品質を保証する。
// 通常の `cargo test` では #[ignore] によりスキップされる。
// CI 環境（DATABASE_URL が設定済み）では `cargo test -- --include-ignored` で実行する。

use k1s0_order_server::domain::entity::order::{
    CreateOrder, CreateOrderItem, OrderStatus,
};
use k1s0_order_server::domain::repository::order_repository::OrderRepository;
use k1s0_order_server::infrastructure::database::order_repository::OrderPostgresRepository;
use k1s0_order_server::MIGRATOR;

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
async fn cleanup(pool: &sqlx::PgPool, customer_id: &str) {
    // outbox イベントから削除（注文 UUID で絞り込み）
    let _ = sqlx::query(
        "DELETE FROM outbox_events WHERE aggregate_id IN \
         (SELECT id::text FROM orders WHERE customer_id = $1)",
    )
    .bind(customer_id)
    .execute(pool)
    .await;
    // 注文明細を削除（外部キー制約のため orders より先に削除）
    let _ = sqlx::query(
        "DELETE FROM order_items WHERE order_id IN \
         (SELECT id FROM orders WHERE customer_id = $1)",
    )
    .bind(customer_id)
    .execute(pool)
    .await;
    // 注文本体を削除
    let _ = sqlx::query("DELETE FROM orders WHERE customer_id = $1")
        .bind(customer_id)
        .execute(pool)
        .await;
}

/// 注文の基本 CRUD が実 DB 上で正しく動作することを検証する。
#[tokio::test]
#[ignore = "実 DB テスト: DATABASE_URL 環境変数が必要。CI では --include-ignored で実行する"]
async fn test_order_crud_with_real_db() {
    let pool = setup_test_db().await;
    let repo = OrderPostgresRepository::new(pool.clone());
    let customer_id = "test-customer-db-crud";

    cleanup(&pool, customer_id).await;

    // CREATE: 注文を作成する
    let input = CreateOrder {
        customer_id: customer_id.to_string(),
        currency: "JPY".to_string(),
        notes: None,
        items: vec![CreateOrderItem {
            product_id: "PROD-001".to_string(),
            product_name: "テスト商品".to_string(),
            quantity: 2,
            unit_price: 1500,
        }],
    };
    let (order, items) = repo
        .create(&input, "test-user")
        .await
        .expect("注文の作成に失敗");
    assert_eq!(order.customer_id, customer_id);
    assert_eq!(order.total_amount, 3000); // 2 * 1500
    assert_eq!(order.version, 1);
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].product_id, "PROD-001");

    // READ: ID で注文を取得できる
    let found = repo
        .find_by_id(order.id)
        .await
        .expect("find_by_id に失敗")
        .expect("注文が見つからない");
    assert_eq!(found.id, order.id);
    assert_eq!(found.customer_id, customer_id);
    assert_eq!(found.total_amount, 3000);

    // READ: 注文明細を取得できる
    let found_items = repo
        .find_items_by_order_id(order.id)
        .await
        .expect("find_items_by_order_id に失敗");
    assert_eq!(found_items.len(), 1);
    assert_eq!(found_items[0].product_id, "PROD-001");

    cleanup(&pool, customer_id).await;
}

/// 楽観ロック（version チェック）が実 DB 上で正しく動作することを検証する。
/// ステータス更新時に version が一致しない場合はエラーになることを確認する（R-02 対応）。
#[tokio::test]
#[ignore = "実 DB テスト: DATABASE_URL 環境変数が必要。CI では --include-ignored で実行する"]
async fn test_order_optimistic_lock_with_real_db() {
    let pool = setup_test_db().await;
    let repo = OrderPostgresRepository::new(pool.clone());
    let customer_id = "test-customer-db-optlock";

    cleanup(&pool, customer_id).await;

    // 初期注文を作成する（version=1）
    let input = CreateOrder {
        customer_id: customer_id.to_string(),
        currency: "JPY".to_string(),
        notes: None,
        items: vec![CreateOrderItem {
            product_id: "PROD-002".to_string(),
            product_name: "ロックテスト商品".to_string(),
            quantity: 1,
            unit_price: 1000,
        }],
    };
    let (order, _) = repo
        .create(&input, "test-user")
        .await
        .expect("注文の作成に失敗");
    assert_eq!(order.version, 1);

    // 正常なステータス更新: version=1 で確認済みに変更する
    let updated = repo
        .update_status(order.id, &OrderStatus::Confirmed, "test-user", order.version)
        .await
        .expect("注文ステータス更新に失敗");
    assert_eq!(updated.version, 2); // version がインクリメントされる

    // 楽観ロック違反: 古い version=1 で再度更新を試みるとエラーになる
    let stale_result = repo
        .update_status(order.id, &OrderStatus::Processing, "test-user", 1)
        .await;
    assert!(
        stale_result.is_err(),
        "古い version で更新を試みたときにエラーが発生することを期待する"
    );

    cleanup(&pool, customer_id).await;
}

/// Outbox イベントの挿入とフェッチが実 DB 上で正しく動作することを検証する。
#[tokio::test]
#[ignore = "実 DB テスト: DATABASE_URL 環境変数が必要。CI では --include-ignored で実行する"]
async fn test_order_outbox_events_with_real_db() {
    let pool = setup_test_db().await;
    let repo = OrderPostgresRepository::new(pool.clone());
    let customer_id = "test-customer-db-outbox";

    cleanup(&pool, customer_id).await;

    // 注文を作成する（create 内部で Outbox イベントが自動挿入される）
    let input = CreateOrder {
        customer_id: customer_id.to_string(),
        currency: "JPY".to_string(),
        notes: None,
        items: vec![CreateOrderItem {
            product_id: "PROD-003".to_string(),
            product_name: "Outbox テスト商品".to_string(),
            quantity: 1,
            unit_price: 500,
        }],
    };
    let (order, _) = repo
        .create(&input, "test-user")
        .await
        .expect("注文の作成に失敗");

    // 未パブリッシュのイベントを取得できる（create 時に自動挿入済み）
    let events = repo
        .fetch_unpublished_events(10)
        .await
        .expect("fetch_unpublished_events に失敗");
    // 今回の注文に対するイベントが存在すること
    let order_events: Vec<_> = events
        .iter()
        .filter(|e| e.aggregate_id == order.id.to_string())
        .collect();
    assert!(!order_events.is_empty(), "order.created Outbox イベントが取得できること");

    // イベントをパブリッシュ済みとしてマークする
    let event_ids: Vec<_> = order_events.iter().map(|e| e.id).collect();
    repo.mark_events_published(&event_ids)
        .await
        .expect("mark_events_published に失敗");

    // マーク後は同じ注文 ID のイベントが未パブリッシュ一覧に出ないこと
    let remaining = repo
        .fetch_unpublished_events(10)
        .await
        .expect("fetch_unpublished_events に失敗");
    let remaining_order_events: Vec<_> = remaining
        .iter()
        .filter(|e| e.aggregate_id == order.id.to_string())
        .collect();
    assert!(
        remaining_order_events.is_empty(),
        "パブリッシュ済みイベントは取得されないこと"
    );

    cleanup(&pool, customer_id).await;
}
