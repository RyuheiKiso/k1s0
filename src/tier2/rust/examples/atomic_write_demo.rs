// k1s0 tier2 atomic_write_demo: AtomicTripleWrite + TenantContext のデモ
// 検収コマンド: cargo run --example atomic_write_demo
//
// P1-P4 invariant を mock（スタブ）で検証するデモプログラム
// TEST_DATABASE_URL が設定されていない場合は SQL 生成デモを実行する
// TEST_DATABASE_URL が設定されている場合は実 PostgreSQL に接続して三表書込を実行する

// k1s0_tier2 の公開 API を全て import する
use k1s0_tier2::{
    // AtomicTripleWrite: P1-P4 atomic 書込エンジン
    AtomicTripleWrite,
    // TenantContext: テナントコンテキスト
    TenantContext,
    // SessionPurpose: セッション目的
    SessionPurpose,
};
// atomic_triple_write の StateChange / TableClass を import する
use k1s0_tier2::atomic_triple_write::{StateChange, TableClass};
// UUID ライブラリ
use uuid::Uuid;
// 環境変数取得
use std::env;

// tokio 非同期ランタイム
#[tokio::main]
async fn main() {
    // デモ開始メッセージを出力する
    println!("=== k1s0 tier2 AtomicTripleWrite + TenantContext デモ ===");
    println!();

    // ============================================================
    // P3: tenant_id 一致検証のデモ（ビルドが通ることを確認する）
    // ============================================================
    println!("--- P3: cross-tenant isolation デモ ---");

    // テナント A の TenantContext を生成する（AuthContext 経由のみ許容）
    let tenant_a = Uuid::new_v4();
    // TenantContext::from_auth でコンテキストを生成する（API 引数経由は禁止）
    let ctx_a = TenantContext::from_auth(
        tenant_a,
        "demo-actor-a".to_string(),
        SessionPurpose::BusinessOp,
    );
    // AtomicTripleWrite をテナント A のコンテキストで生成する
    let writer_a = AtomicTripleWrite::new(ctx_a);
    // テナント A のコンテキストで正常な StateChange を生成する
    let valid_change = StateChange {
        // テスト用の aggregate ID を生成する
        aggregate_id: Uuid::new_v4(),
        // TenantContext と同一の tenant_id を設定する（P3 OK）
        tenant_id: tenant_a,
        // TenantScoped テーブルクラスを指定する
        table_class: TableClass::TenantScoped,
        // テスト用ペイロードを設定する
        payload: serde_json::json!({"event": "OrderCreated", "status": "Draft"}),
        // バージョン 1 で開始する
        version: 1,
    };
    // P3: tenant_id 一致検証（成功するはず）
    match writer_a.verify_tenant_id(&valid_change) {
        // 検証成功: tenant_id が一致した場合
        Ok(_) => println!("  P3 [OK]: tenant_id 一致 - 書込みを許可する"),
        // 検証失敗: tenant_id が不一致の場合（このデモでは発生しないはず）
        Err(e) => println!("  P3 [UNEXPECTED ERROR]: {}", e),
    }

    // テナント B の tenant_id で cross-tenant 攻撃を模擬する
    let tenant_b = Uuid::new_v4();
    // テナント A のコンテキストに テナント B の StateChange を渡す（P3 違反）
    let cross_tenant_change = StateChange {
        aggregate_id: Uuid::new_v4(),
        // テナント A のコンテキストで テナント B の tenant_id を使う（P3 違反）
        tenant_id: tenant_b,
        table_class: TableClass::TenantScoped,
        payload: serde_json::json!({"cross_tenant": "attack_attempt"}),
        version: 1,
    };
    // P3: cross-tenant の StateChange が即座に reject されることを確認する
    match writer_a.verify_tenant_id(&cross_tenant_change) {
        // P3 違反が正しく検出されるはず
        Ok(_) => println!("  P3 [UNEXPECTED OK]: cross-tenant write が許可された（バグ）"),
        // P3 違反: reject されることを確認する
        Err(e) => println!("  P3 [OK]: cross-tenant write を reject: {}", e),
    }

    println!();

    // ============================================================
    // P4: pii_segregated audit 必須フラグのデモ
    // ============================================================
    println!("--- P4: PII segregated audit デモ ---");

    // PII アクセス用の TenantContext を生成する（Support purpose）
    let ctx_support = TenantContext::from_auth(
        tenant_a,
        "support-engineer-001".to_string(),
        // Support purpose: PII 参照時のセッション目的
        SessionPurpose::Support,
    );
    // AtomicTripleWrite を Support コンテキストで生成する
    let writer_support = AtomicTripleWrite::new(ctx_support);
    // P4: PiiSegregated テーブルクラスの StateChange を生成する
    let pii_change = StateChange {
        aggregate_id: Uuid::new_v4(),
        tenant_id: tenant_a,
        // PiiSegregated を指定する（P4 audit 必須フラグが true になる）
        table_class: TableClass::PiiSegregated,
        // PII フィールドは redact 済みのみ含む
        payload: serde_json::json!({"pii_field": "[REDACTED]"}),
        version: 1,
    };
    // P4: pii_required フラグの確認
    let pii_required = writer_support.verify_pii_audit_required(&pii_change);
    // PiiSegregated では audit が必須であることを確認する
    println!(
        "  P4: PiiSegregated audit required = {} (expected: true)",
        pii_required
    );
    // TenantScoped は audit 任意（pgaudit 不要）
    let normal_change = StateChange {
        aggregate_id: Uuid::new_v4(),
        tenant_id: tenant_a,
        table_class: TableClass::TenantScoped,
        payload: serde_json::json!({"event": "StateChange"}),
        version: 1,
    };
    let normal_audit_required = writer_support.verify_pii_audit_required(&normal_change);
    // TenantScoped は pgaudit 不要（audit_event は必ず書くが pgaudit は不要）
    println!(
        "  P4: TenantScoped audit required = {} (expected: false)",
        normal_audit_required
    );

    println!();

    // ============================================================
    // P1: SQL 生成デモ（実 DB 不要）
    // ============================================================
    println!("--- P1: triple write SQL 生成デモ ---");

    // TenantContext の SET LOCAL SQL を確認する
    let ctx_demo = TenantContext::from_auth(
        tenant_a,
        "demo-actor-sql".to_string(),
        SessionPurpose::BusinessOp,
    );
    // AtomicTripleWrite を生成する
    let writer_demo = AtomicTripleWrite::new(ctx_demo);
    // SQL を生成する（実 DB 不要）
    match writer_demo.build_triple_write_sql(&valid_change) {
        Ok(sql) => {
            // SQL が生成されたことを確認する
            println!("  P1: triple write SQL 生成成功 ({} chars)", sql.len());
            // BEGIN / domain_event / outbox_message / audit_event / COMMIT が含まれることを確認する
            println!("    - BEGIN:         {}", sql.contains("BEGIN"));
            println!("    - domain_event:  {}", sql.contains("domain_event"));
            // outbox_message テーブル名を確認する（migration SoT: k1s0.outbox_message）
            println!("    - outbox_message:{}", sql.contains("outbox_message"));
            println!("    - audit_event:   {}", sql.contains("audit_event"));
            println!("    - COMMIT:       {}", sql.contains("COMMIT"));
            println!("    - app.tenant_id GUC 注入: {}", sql.contains("app.tenant_id"));
        }
        Err(e) => {
            // SQL 生成失敗（P3 違反等）
            println!("  P1: SQL 生成失敗: {}", e);
        }
    }

    println!();

    // ============================================================
    // TEST_DATABASE_URL が設定されている場合は実 PostgreSQL 書込デモを実行する
    // ============================================================
    match env::var("TEST_DATABASE_URL") {
        Ok(db_url) => {
            // 実 PostgreSQL 書込デモを実行する
            println!("--- 実 PostgreSQL 書込デモ (TEST_DATABASE_URL 設定済み) ---");
            // PostgreSQL 接続プールを生成する
            match sqlx::postgres::PgPoolOptions::new()
                .max_connections(2)
                .connect(&db_url)
                .await
            {
                Ok(pool) => {
                    // テスト用スキーマをセットアップする
                    let setup_result = sqlx::query(r#"
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
                    "#).execute(&pool).await;
                    // スキーマセットアップ結果を確認する
                    match setup_result {
                        Ok(_) => println!("  スキーマセットアップ成功"),
                        Err(e) => {
                            println!("  スキーマセットアップ失敗（既存スキーマかもしれない）: {}", e);
                        }
                    }
                    // P1: 実 PostgreSQL 三表書込を実行する
                    let ctx_pg = TenantContext::from_auth(
                        tenant_a,
                        "demo-actor-pg".to_string(),
                        SessionPurpose::BusinessOp,
                    );
                    let writer_pg = AtomicTripleWrite::new(ctx_pg);
                    let pg_change = StateChange {
                        aggregate_id: Uuid::new_v4(),
                        tenant_id: tenant_a,
                        table_class: TableClass::TenantScoped,
                        payload: serde_json::json!({"demo": "atomic_triple_write_demo"}),
                        version: 1,
                    };
                    // transaction を開く（P1 の atomic 三表書込に使用する）
                    match pool.begin().await {
                        Ok(mut tx) => {
                            // P1: execute() を呼んで 3 テーブルに INSERT する
                            match writer_pg.execute(&pg_change, &mut tx).await {
                                Ok(result) => {
                                    // transaction を commit する
                                    if let Err(e) = tx.commit().await {
                                        println!("  P1: commit 失敗: {}", e);
                                    } else {
                                        // 書込成功メッセージを出力する
                                        println!("  P1: 三表書込成功 (aggregate_id={}, outbox_id={}, audit_id={})",
                                            result.aggregate_id,
                                            result.outbox_id,
                                            result.audit_event_id
                                        );
                                    }
                                }
                                Err(e) => {
                                    // execute 失敗の場合は rollback する
                                    let _ = tx.rollback().await;
                                    println!("  P1: 三表書込失敗: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            // transaction 開始失敗
                            println!("  P1: transaction 開始失敗: {}", e);
                        }
                    }
                    // 接続プールを閉じる
                    pool.close().await;
                }
                Err(e) => {
                    // PostgreSQL 接続失敗
                    println!("  PostgreSQL 接続失敗: {}", e);
                }
            }
        }
        Err(_) => {
            // TEST_DATABASE_URL 未設定: SQL 生成デモのみ実行した旨を出力する
            println!("--- 実 PostgreSQL 書込デモをスキップ (TEST_DATABASE_URL 未設定) ---");
            println!("  実行するには:");
            println!("  TEST_DATABASE_URL=postgres://k1s0:k1s0dev@localhost/tier2_dev cargo run --example atomic_write_demo");
        }
    }

    println!();
    println!("=== デモ完了 ===");
}
