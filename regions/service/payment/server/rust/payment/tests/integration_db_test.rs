// 決済サーバーの実 DB 統合テスト（T-01/T-02/R-03 対応）
// このテストは実際の PostgreSQL に接続し、DB レイヤーの品質を保証する。
// 通常の `cargo test` では #[ignore] によりスキップされる。
// CI 環境（DATABASE_URL が設定済み）では `cargo test -- --include-ignored` で実行する。

use k1s0_payment_server::domain::entity::payment::InitiatePayment;
use k1s0_payment_server::domain::repository::payment_repository::PaymentRepository;
use k1s0_payment_server::infrastructure::database::payment_repository::PaymentPostgresRepository;
use k1s0_payment_server::MIGRATOR;

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
async fn cleanup(pool: &sqlx::PgPool, order_id: &str) {
    // outbox イベントから削除（決済 UUID で絞り込み）
    let _ = sqlx::query(
        "DELETE FROM outbox_events WHERE aggregate_id IN \
         (SELECT id::text FROM payments WHERE order_id = $1)",
    )
    .bind(order_id)
    .execute(pool)
    .await;
    // 決済レコードを削除
    let _ = sqlx::query("DELETE FROM payments WHERE order_id = $1")
        .bind(order_id)
        .execute(pool)
        .await;
}

/// 決済の基本 CRUD が実 DB 上で正しく動作することを検証する。
#[tokio::test]
#[ignore = "実 DB テスト: DATABASE_URL 環境変数が必要。CI では --include-ignored で実行する"]
async fn test_payment_crud_with_real_db() {
    let pool = setup_test_db().await;
    let repo = PaymentPostgresRepository::new(pool.clone());
    let order_id = "test-order-db-crud-001";

    cleanup(&pool, order_id).await;

    // CREATE: 決済を開始する
    let input = InitiatePayment {
        order_id: order_id.to_string(),
        customer_id: "test-customer-001".to_string(),
        amount: 5000,
        currency: "JPY".to_string(),
        payment_method: Some("credit_card".to_string()),
    };
    let payment = repo
        .create(&input)
        .await
        .expect("決済の作成に失敗");
    assert_eq!(payment.order_id, order_id);
    assert_eq!(payment.amount, 5000);
    assert_eq!(payment.version, 1);

    // READ: ID で決済を取得できる
    let found = repo
        .find_by_id(payment.id)
        .await
        .expect("find_by_id に失敗")
        .expect("決済が見つからない");
    assert_eq!(found.id, payment.id);
    assert_eq!(found.order_id, order_id);
    assert_eq!(found.amount, 5000);

    // READ: 注文 ID で決済を取得できる（冪等性チェック用）
    let found_by_order = repo
        .find_by_order_id(order_id)
        .await
        .expect("find_by_order_id に失敗")
        .expect("注文 ID で決済が見つからない");
    assert_eq!(found_by_order.id, payment.id);

    cleanup(&pool, order_id).await;
}

/// 楽観ロック（version チェック）が実 DB 上で正しく動作することを検証する。
/// 決済完了時に version が一致しない場合はエラーになることを確認する（R-02 対応）。
#[tokio::test]
#[ignore = "実 DB テスト: DATABASE_URL 環境変数が必要。CI では --include-ignored で実行する"]
async fn test_payment_optimistic_lock_with_real_db() {
    let pool = setup_test_db().await;
    let repo = PaymentPostgresRepository::new(pool.clone());
    let order_id = "test-order-db-optlock-001";

    cleanup(&pool, order_id).await;

    // 初期決済を作成する（version=1）
    let input = InitiatePayment {
        order_id: order_id.to_string(),
        customer_id: "test-customer-001".to_string(),
        amount: 3000,
        currency: "JPY".to_string(),
        payment_method: Some("credit_card".to_string()),
    };
    let payment = repo
        .create(&input)
        .await
        .expect("決済の作成に失敗");
    assert_eq!(payment.version, 1);

    // 正常な決済完了: version=1 でトランザクション ID を設定する
    let completed = repo
        .complete(payment.id, "txn-test-001", payment.version)
        .await
        .expect("決済完了に失敗");
    assert_eq!(completed.version, 2); // version がインクリメントされる

    // 楽観ロック違反: 古い version=1 で再度完了を試みるとエラーになる
    let stale_result = repo
        .complete(payment.id, "txn-test-002", 1)
        .await;
    assert!(
        stale_result.is_err(),
        "古い version で完了を試みたときにエラーが発生することを期待する"
    );

    cleanup(&pool, order_id).await;
}

/// 決済冪等性が実 DB 上で正しく動作することを検証する（R-03 対応）。
/// 同一 order_id で重複して決済を作成した場合、UNIQUE 制約により既存の決済が返されることを確認する。
#[tokio::test]
#[ignore = "実 DB テスト: DATABASE_URL 環境変数が必要。CI では --include-ignored で実行する"]
async fn test_payment_idempotency_with_real_db() {
    let pool = setup_test_db().await;
    let repo = PaymentPostgresRepository::new(pool.clone());
    let order_id = "test-order-db-idempotency-001";

    cleanup(&pool, order_id).await;

    // 最初の決済を作成する
    let input = InitiatePayment {
        order_id: order_id.to_string(),
        customer_id: "test-customer-001".to_string(),
        amount: 2000,
        currency: "JPY".to_string(),
        payment_method: Some("credit_card".to_string()),
    };
    let first_payment = repo
        .create(&input)
        .await
        .expect("1回目の決済作成に失敗");

    // 同一 order_id で2回目の決済を試みる（ON CONFLICT DO NOTHING → 既存レコードを返す）
    let duplicate_input = InitiatePayment {
        order_id: order_id.to_string(),
        customer_id: "test-customer-001".to_string(),
        amount: 9999, // 異なる金額でも既存決済が返されること
        currency: "JPY".to_string(),
        payment_method: Some("bank_transfer".to_string()),
    };
    let second_payment = repo
        .create(&duplicate_input)
        .await
        .expect("2回目の決済作成に失敗（冪等性: 既存決済が返されること）");

    // 同一の決済 ID が返されることを確認する（冪等性保証）
    assert_eq!(
        first_payment.id, second_payment.id,
        "同一 order_id の重複決済は既存の決済 ID を返すこと"
    );
    // 最初の金額が保持されること（上書きされないこと）
    assert_eq!(
        second_payment.amount, 2000,
        "冪等性により最初の決済金額が保持されること"
    );

    // DB に決済レコードが 1 件のみ存在することを確認する
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM payments WHERE order_id = $1")
        .bind(order_id)
        .fetch_one(&pool)
        .await
        .expect("決済件数の取得に失敗");
    assert_eq!(count, 1, "同一 order_id の決済レコードは 1 件のみ存在すること");

    cleanup(&pool, order_id).await;
}

/// Outbox イベントの挿入とフェッチが実 DB 上で正しく動作することを検証する。
#[tokio::test]
#[ignore = "実 DB テスト: DATABASE_URL 環境変数が必要。CI では --include-ignored で実行する"]
async fn test_payment_outbox_events_with_real_db() {
    let pool = setup_test_db().await;
    let repo = PaymentPostgresRepository::new(pool.clone());
    let order_id = "test-order-db-outbox-001";

    cleanup(&pool, order_id).await;

    // 決済を作成する（create 内部で Outbox イベントが自動挿入される）
    let input = InitiatePayment {
        order_id: order_id.to_string(),
        customer_id: "test-customer-001".to_string(),
        amount: 1000,
        currency: "JPY".to_string(),
        payment_method: None,
    };
    let payment = repo
        .create(&input)
        .await
        .expect("決済の作成に失敗");

    // 未パブリッシュのイベントを取得できる（create 時に自動挿入済み）
    let events = repo
        .fetch_unpublished_events(10)
        .await
        .expect("fetch_unpublished_events に失敗");
    // 今回の決済に対するイベントが存在すること
    let payment_events: Vec<_> = events
        .iter()
        .filter(|e| e.aggregate_id == payment.id.to_string())
        .collect();
    assert!(
        !payment_events.is_empty(),
        "payment.initiated Outbox イベントが取得できること"
    );

    // イベントをパブリッシュ済みとしてマークする
    let event_ids: Vec<_> = payment_events.iter().map(|e| e.id).collect();
    repo.mark_events_published(&event_ids)
        .await
        .expect("mark_events_published に失敗");

    // マーク後は同じ決済 ID のイベントが未パブリッシュ一覧に出ないこと
    let remaining = repo
        .fetch_unpublished_events(10)
        .await
        .expect("fetch_unpublished_events に失敗");
    let remaining_payment_events: Vec<_> = remaining
        .iter()
        .filter(|e| e.aggregate_id == payment.id.to_string())
        .collect();
    assert!(
        remaining_payment_events.is_empty(),
        "パブリッシュ済みイベントは取得されないこと"
    );

    cleanup(&pool, order_id).await;
}
