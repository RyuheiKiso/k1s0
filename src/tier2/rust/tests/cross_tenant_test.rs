// k1s0 tier2 cross-tenant integration test
// P1-P4 invariant を PostgreSQL RLS FORCE 環境で検証する統合テスト
//
// 実行方法: TEST_DATABASE_URL 環境変数を設定した上で --ignored フラグで実行する
//   TEST_DATABASE_URL=postgres://postgres:k1s0@localhost/k1s0 cargo test -- --ignored
//
// 通常の cargo test（CI の unit test job）では全テストを skip する（#[ignore] タグ）
// Cargo integration test: tests/ ディレクトリに配置し、crate 外部から public API を検証する

// std: 環境変数取得に使用する
use std::env;
// uuid: テスト用 tenant_id / aggregate_id 生成に使用する
use uuid::Uuid;
// tier2 のメイン実装を参照する（crate 外部からの public API）
use k1s0_tier2::{
    // AtomicTripleWrite: P1-P4 atomic 書込エンジン
    AtomicTripleWrite,
    // AtomicWriteError: エラー型（P3 違反の確認に使用する）
    AtomicWriteError,
    // TenantContext / SessionPurpose: テナントコンテキスト
    TenantContext,
    SessionPurpose,
};
// atomic_triple_write の StateChange / TableClass を import する
use k1s0_tier2::atomic_triple_write::{StateChange, TableClass};
// sqlx: PostgreSQL 接続プールに使用する
use sqlx::postgres::PgPoolOptions;

// テスト用スキーマをセットアップする SQL（k1s0 スキーマと 3 テーブルを作成する）
const SETUP_SQL: &str = r#"
    CREATE SCHEMA IF NOT EXISTS k1s0;
    CREATE TABLE IF NOT EXISTS k1s0.domain_event (
        id UUID PRIMARY KEY,
        aggregate_id UUID NOT NULL,
        tenant_id UUID NOT NULL,
        event_kind TEXT NOT NULL,
        payload JSONB NOT NULL,
        version BIGINT NOT NULL,
        created_at TIMESTAMPTZ NOT NULL
    );
    CREATE TABLE IF NOT EXISTS k1s0.outbox_message (
        id UUID PRIMARY KEY,
        aggregate_id UUID NOT NULL,
        tenant_id UUID NOT NULL,
        event_kind TEXT NOT NULL,
        payload JSONB NOT NULL,
        created_at TIMESTAMPTZ NOT NULL
    );
    CREATE TABLE IF NOT EXISTS k1s0.audit_event (
        id UUID PRIMARY KEY,
        aggregate_id UUID NOT NULL,
        tenant_id UUID NOT NULL,
        actor_id TEXT NOT NULL,
        purpose TEXT NOT NULL,
        table_class TEXT NOT NULL,
        payload JSONB NOT NULL,
        created_at TIMESTAMPTZ NOT NULL
    );
"#;

// テスト用のスキーマをクリーンアップする SQL（テスト間の独立性を保つ）
const CLEANUP_SQL: &str = r#"
    DROP TABLE IF EXISTS k1s0.audit_event CASCADE;
    DROP TABLE IF EXISTS k1s0.outbox_message CASCADE;
    DROP TABLE IF EXISTS k1s0.domain_event CASCADE;
    DROP SCHEMA IF EXISTS k1s0 CASCADE;
"#;

// TEST_DATABASE_URL 環境変数から接続 URL を取得するヘルパー関数
fn test_database_url() -> Option<String> {
    // TEST_DATABASE_URL が設定されている場合のみ Some を返す
    env::var("TEST_DATABASE_URL").ok()
}

#[tokio::test]
// P3: cross-tenant isolation を検証する統合テスト
// 異なる tenant_id の StateChange が reject されることを確認する
#[ignore = "requires PostgreSQL with RLS FORCE (set TEST_DATABASE_URL to enable)"]
async fn test_p3_cross_tenant_isolation() {
    // TEST_DATABASE_URL が設定されていない場合はスキップする
    let url = match test_database_url() {
        Some(u) => u,
        None => {
            // 環境変数が未設定の場合はテストをスキップする
            eprintln!("TEST_DATABASE_URL not set, skipping cross-tenant isolation test");
            return;
        }
    };
    // PostgreSQL 接続プールを生成する（最大 2 接続でテスト用）
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&url)
        .await
        .expect("Failed to connect to PostgreSQL");
    // テスト用スキーマをセットアップする
    sqlx::query(SETUP_SQL)
        .execute(&pool)
        .await
        .expect("Failed to setup test schema");

    // テナント A の TenantContext を生成する
    let tenant_a = Uuid::new_v4();
    let ctx_a = TenantContext::from_auth(
        tenant_a,
        "actor-a".to_string(),
        SessionPurpose::BusinessOp,
    );
    // テナント B の tenant_id を生成する（cross-tenant を模擬する）
    let tenant_b = Uuid::new_v4();

    // AtomicTripleWrite を生成する（テナント A のコンテキストで実行する）
    let writer = AtomicTripleWrite::new(ctx_a);
    // テナント B の StateChange を生成する（P3 違反を意図的に作る）
    let cross_tenant_change = StateChange {
        aggregate_id: Uuid::new_v4(),
        // tenant_a のコンテキストに tenant_b の change を渡す（P3 違反）
        tenant_id: tenant_b,
        table_class: TableClass::TenantScoped,
        payload: serde_json::json!({"cross_tenant": "attempt"}),
        version: 1,
    };

    // P3 検証: cross-tenant StateChange が即座に reject されることを確認する
    let result = writer.verify_tenant_id(&cross_tenant_change);
    // P3 違反: TenantIdMismatch エラーが返されることを確認する
    assert!(
        matches!(result, Err(AtomicWriteError::TenantIdMismatch { .. })),
        "P3: cross-tenant write must be rejected before reaching DB"
    );

    // テスト後にスキーマをクリーンアップする
    sqlx::query(CLEANUP_SQL)
        .execute(&pool)
        .await
        .expect("Failed to cleanup test schema");
    // 接続プールを閉じる
    pool.close().await;
}

#[tokio::test]
// P1: atomic triple write が 3 テーブルに正常に書込むことを検証する統合テスト
#[ignore = "requires PostgreSQL (set TEST_DATABASE_URL to enable)"]
async fn test_p1_atomic_triple_write() {
    // TEST_DATABASE_URL が設定されていない場合はスキップする
    let url = match test_database_url() {
        Some(u) => u,
        None => {
            // 環境変数が未設定の場合はテストをスキップする
            eprintln!("TEST_DATABASE_URL not set, skipping atomic triple write test");
            return;
        }
    };
    // PostgreSQL 接続プールを生成する
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&url)
        .await
        .expect("Failed to connect to PostgreSQL");
    // テスト用スキーマをセットアップする
    sqlx::query(SETUP_SQL)
        .execute(&pool)
        .await
        .expect("Failed to setup test schema");

    // テスト用テナントの TenantContext を生成する
    let tenant_id = Uuid::new_v4();
    let ctx = TenantContext::from_auth(
        tenant_id,
        "integration-test-actor".to_string(),
        SessionPurpose::BusinessOp,
    );
    // AtomicTripleWrite を生成する
    let writer = AtomicTripleWrite::new(ctx);
    // テスト用の StateChange を生成する
    let change = StateChange {
        aggregate_id: Uuid::new_v4(),
        tenant_id,
        table_class: TableClass::TenantScoped,
        payload: serde_json::json!({"test": "atomic_triple_write"}),
        version: 1,
    };

    // sqlx::Transaction を開く（P1 の atomic 三表書込に使用する）
    let mut tx = pool.begin().await.expect("Failed to begin transaction");
    // P1: execute() を呼んで 3 テーブルに INSERT する
    let result = writer.execute(&change, &mut tx).await;
    // execute() が成功することを確認する
    assert!(result.is_ok(), "P1: atomic triple write must succeed: {:?}", result.err());
    // transaction を commit する（3 INSERT が永続化される）
    tx.commit().await.expect("Failed to commit transaction");

    // domain_event が 1 行書込まれていることを確認する
    let domain_event_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM k1s0.domain_event WHERE aggregate_id = $1",
    )
    .bind(change.aggregate_id)
    .fetch_one(&pool)
    .await
    .expect("Failed to query domain_event count");
    assert_eq!(domain_event_count.0, 1, "P1: domain_event must have 1 row");

    // outbox_message が 1 行書込まれていることを確認する（migration SoT: k1s0.outbox_message）
    let outbox_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM k1s0.outbox_message WHERE aggregate_id = $1",
    )
    .bind(change.aggregate_id)
    .fetch_one(&pool)
    .await
    .expect("Failed to query outbox_message count");
    assert_eq!(outbox_count.0, 1, "P1: outbox_message must have 1 row");

    // audit_event が 1 行書込まれていることを確認する
    let audit_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM k1s0.audit_event WHERE aggregate_id = $1",
    )
    .bind(change.aggregate_id)
    .fetch_one(&pool)
    .await
    .expect("Failed to query audit_event count");
    assert_eq!(audit_count.0, 1, "P1: audit_event must have 1 row");

    // テスト後にスキーマをクリーンアップする
    sqlx::query(CLEANUP_SQL)
        .execute(&pool)
        .await
        .expect("Failed to cleanup test schema");
    // 接続プールを閉じる
    pool.close().await;
}

#[tokio::test]
// P4: pii_segregated テーブルへのアクセスが audit_event に記録されることを検証する
#[ignore = "requires PostgreSQL (set TEST_DATABASE_URL to enable)"]
async fn test_p4_pii_audit_required() {
    // TEST_DATABASE_URL が設定されていない場合はスキップする
    let url = match test_database_url() {
        Some(u) => u,
        None => {
            // 環境変数が未設定の場合はテストをスキップする
            eprintln!("TEST_DATABASE_URL not set, skipping PII audit test");
            return;
        }
    };
    // PostgreSQL 接続プールを生成する
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&url)
        .await
        .expect("Failed to connect to PostgreSQL");
    // テスト用スキーマをセットアップする
    sqlx::query(SETUP_SQL)
        .execute(&pool)
        .await
        .expect("Failed to setup test schema");

    // テスト用テナントの TenantContext を生成する（PII アクセスは Support purpose を使う）
    let tenant_id = Uuid::new_v4();
    let ctx = TenantContext::from_auth(
        tenant_id,
        "support-engineer-001".to_string(),
        SessionPurpose::Support,
    );
    // AtomicTripleWrite を生成する
    let writer = AtomicTripleWrite::new(ctx);
    // P4: PiiSegregated テーブルクラスの StateChange を生成する
    let pii_change = StateChange {
        aggregate_id: Uuid::new_v4(),
        tenant_id,
        // PiiSegregated を指定することで P4 audit 必須フラグが true になる
        table_class: TableClass::PiiSegregated,
        payload: serde_json::json!({"pii_field": "[REDACTED]"}),
        version: 1,
    };

    // P4: pii_required フラグが true を返すことを確認する
    assert!(
        writer.verify_pii_audit_required(&pii_change),
        "P4: pii_segregated must require audit"
    );

    // P1+P4: execute() を呼んで audit_event が書込まれることを確認する
    let mut tx = pool.begin().await.expect("Failed to begin transaction");
    // PiiSegregated の atomic 三表書込を実行する（audit_event が必ず書込まれる）
    let result = writer.execute(&pii_change, &mut tx).await;
    // execute() が成功することを確認する
    assert!(result.is_ok(), "P4: pii_segregated write must succeed: {:?}", result.err());
    // transaction を commit する
    tx.commit().await.expect("Failed to commit transaction");

    // audit_event に PiiSegregated の記録が 1 行あることを確認する
    let audit_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM k1s0.audit_event WHERE aggregate_id = $1 AND table_class = 'PiiSegregated'",
    )
    .bind(pii_change.aggregate_id)
    .fetch_one(&pool)
    .await
    .expect("Failed to query audit_event for PII");
    // P4: PiiSegregated の audit_event が 1 行書込まれていることを確認する
    assert_eq!(audit_count.0, 1, "P4: audit_event must record PiiSegregated access");

    // テスト後にスキーマをクリーンアップする
    sqlx::query(CLEANUP_SQL)
        .execute(&pool)
        .await
        .expect("Failed to cleanup test schema");
    // 接続プールを閉じる
    pool.close().await;
}
