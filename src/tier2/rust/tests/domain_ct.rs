//! domain_ct.rs — Domain Event / Aggregate の contract test (integration feature gate)
//! 環境構築 07 検収コマンド: cargo test --test domain_ct --features integration
//!
//! integration feature gate で保護されているため、通常の unit test 実行では skip される。
//! CI の integration job では --features integration を指定して実行する。

// integration feature gate: このファイル全体を integration 専用コードとして保護する
#![cfg(feature = "integration")]

// テスト対象: k1s0_tier2 crate の公開 API を contract test で検証する
use k1s0_tier2::{
    // AtomicTripleWrite: P1-P4 atomic 書込エンジン（domain event contract の中心）
    AtomicTripleWrite,
    // TenantContext: テナントコンテキスト（domain event の tenant_id 整合性保証に使用する）
    TenantContext,
    // SessionPurpose: セッション目的（domain event の purpose 整合性保証に使用する）
    SessionPurpose,
};
// StateChange / TableClass を atomic_triple_write サブモジュールから import する
use k1s0_tier2::atomic_triple_write::{StateChange, TableClass};
// UUID: テスト用 aggregate_id / tenant_id 生成に使用する
use uuid::Uuid;
// sqlx::PgPool: contract test ではオフライン用 connect_lazy でダミープールを生成する
use sqlx::PgPool;

// contract test 用のオフラインダミー PgPool を生成するヘルパー関数（DB 接続は不要）
fn make_test_pool() -> PgPool {
    // TEST_DATABASE_URL が設定されている場合はその URL を使用する（未設定時はダミー URL）
    let url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/k1s0_test".to_string());
    // connect_lazy: 実際の接続を遅延させてオフライン contract test でも PgPool を生成できるようにする
    PgPool::connect_lazy(&url)
        .expect("connect_lazy should not fail on valid URL format")
}

// テスト用の TenantContext を生成するヘルパー関数（integration test 共通）
// tenant_id: テスト用テナント識別子
fn make_ctx(tenant_id: Uuid) -> TenantContext {
    // from_auth: API 引数経由の tenant_id 渡しを禁止するための唯一の生成経路
    TenantContext::from_auth(
        // 認証済みテナント ID（integration test 用 UUID v4）
        tenant_id,
        // 検収コマンド用テスト actor ID
        "domain-ct-test-actor".to_string(),
        // 通常業務操作（contract test のデフォルト purpose）
        SessionPurpose::BusinessOp,
    )
}

// contract: TenantScoped aggregate の StateChange が P3 tenant_id 検証を通過することを検証する
// (build_triple_write_sql は raw SQL concat 禁止規律により削除済み: 三表書込は execute() の sqlx::query で実施)
#[test]
fn ct_tenant_scoped_aggregate_state_change() {
    // テスト用テナント ID を生成する
    let tenant_id = Uuid::new_v4();
    // TenantContext を生成する
    let ctx = make_ctx(tenant_id);
    // テスト用オフラインダミー PgPool を生成する
    let pool = make_test_pool();
    // AtomicTripleWrite を生成する（TenantContext と PgPool を渡す）
    let writer = AtomicTripleWrite::new(ctx, pool);
    // TenantScoped aggregate の StateChange を生成する
    let change = StateChange {
        // テスト用 aggregate ID
        aggregate_id: Uuid::new_v4(),
        // テスト用テナント ID（TenantContext と一致させる）
        tenant_id,
        // TenantScoped: テナント境界内のデータ書込（RLS FORCE が tenant_id を検証する）
        table_class: TableClass::TenantScoped,
        // テスト用ペイロード（業務データを模したサンプル）
        payload: serde_json::json!({
            "domain_event": "ResourceStatusChanged",
            "aggregate_class": "ResourceUnit",
            "new_status": "RESOURCE_STATUS_MAINTENANCE"
        }),
        // 楽観的ロックバージョン（初期値 1）
        version: 1,
    };

    // contract: verify_tenant_id が matching tenant_id で Ok を返すこと（P3 invariant）
    assert!(
        writer.verify_tenant_id(&change).is_ok(),
        "ct: TenantScoped StateChange must pass verify_tenant_id"
    );
    // contract: TenantScoped は pii_audit_required が false であること（P4 invariant）
    assert!(
        !writer.verify_pii_audit_required(&change),
        "ct: TenantScoped must not require pii audit flag"
    );
}

// contract: TenantMaster aggregate の StateChange が P3 tenant_id 検証を通過することを検証する
// (build_triple_write_sql は raw SQL concat 禁止規律により削除済み)
#[test]
fn ct_tenant_master_aggregate_state_change() {
    // テスト用テナント ID を生成する
    let tenant_id = Uuid::new_v4();
    // TenantContext を生成する
    let ctx = make_ctx(tenant_id);
    // テスト用オフラインダミー PgPool を生成する
    let pool = make_test_pool();
    // AtomicTripleWrite を生成する（TenantContext と PgPool を渡す）
    let writer = AtomicTripleWrite::new(ctx, pool);
    // TenantMaster aggregate の StateChange を生成する
    let change = StateChange {
        // テスト用 aggregate ID
        aggregate_id: Uuid::new_v4(),
        // テスト用テナント ID
        tenant_id,
        // TenantMaster: テナントマスタデータ（role 制限付き RLS FORCE）
        table_class: TableClass::TenantMaster,
        // テスト用ペイロード（テナントマスタ更新を模したサンプル）
        payload: serde_json::json!({
            "domain_event": "TenantMasterUpdated",
            "aggregate_class": "TenantMaster",
            "changed_field": "quota_class"
        }),
        // 楽観的ロックバージョン
        version: 2,
    };

    // contract: verify_tenant_id が matching tenant_id で Ok を返すこと（P3 invariant）
    assert!(
        writer.verify_tenant_id(&change).is_ok(),
        "ct: TenantMaster StateChange must pass verify_tenant_id"
    );
    // contract: TenantMaster も pii_audit_required は false であること（P4 invariant）
    assert!(
        !writer.verify_pii_audit_required(&change),
        "ct: TenantMaster must not require pii audit flag"
    );
}

// contract: PiiSegregated aggregate の StateChange が audit 必須フラグを設定することを検証する
#[test]
fn ct_pii_segregated_aggregate_requires_audit() {
    // テスト用テナント ID を生成する（PII アクセスは Support purpose を使用する）
    let tenant_id = Uuid::new_v4();
    // Support purpose の TenantContext を生成する（PII アクセスは Support role が必要）
    let ctx = TenantContext::from_auth(
        tenant_id,
        "support-engineer-ct-001".to_string(),
        // Support: PII 参照操作に使用する purpose
        SessionPurpose::Support,
    );
    // テスト用オフラインダミー PgPool を生成する
    let pool = make_test_pool();
    // AtomicTripleWrite を生成する（TenantContext と PgPool を渡す）
    let writer = AtomicTripleWrite::new(ctx, pool);
    // PiiSegregated aggregate の StateChange を生成する（P4 audit 必須の対象）
    let pii_change = StateChange {
        aggregate_id: Uuid::new_v4(),
        tenant_id,
        // PiiSegregated: PII 含有テーブル（pgaudit + アプリ層 audit_event の両方が必須）
        table_class: TableClass::PiiSegregated,
        // PII フィールドは redact 済みのみペイロードに含む（raw PII は禁止）
        payload: serde_json::json!({
            "domain_event": "PiiFieldAccessed",
            "pii_field_name": "personal_name",
            "value": "[REDACTED]"
        }),
        version: 1,
    };

    // contract: PiiSegregated では verify_pii_audit_required が true を返すこと（P4 invariant）
    assert!(
        writer.verify_pii_audit_required(&pii_change),
        "ct: PiiSegregated aggregate must require audit (P4 invariant)"
    );
    // contract: PiiSegregated の P3 tenant_id 検証も通過すること（三表書込の前提条件）
    // (build_triple_write_sql は raw SQL concat 禁止規律により削除済み)
    assert!(
        writer.verify_tenant_id(&pii_change).is_ok(),
        "ct: PiiSegregated StateChange must pass verify_tenant_id"
    );
}

// contract: 異なるテナントの StateChange が reject されることを検証する（P3 cross-tenant contract）
#[test]
fn ct_cross_tenant_state_change_rejected() {
    // テナント A の TenantContext を生成する
    let tenant_a = Uuid::new_v4();
    let ctx_a = make_ctx(tenant_a);
    // テスト用オフラインダミー PgPool を生成する
    let pool = make_test_pool();
    // AtomicTripleWrite はテナント A のコンテキストで生成する（TenantContext と PgPool を渡す）
    let writer = AtomicTripleWrite::new(ctx_a, pool);

    // テナント B の StateChange を生成する（cross-tenant 書込の試み）
    let tenant_b = Uuid::new_v4();
    let cross_change = StateChange {
        aggregate_id: Uuid::new_v4(),
        // P3 違反: テナント A のコンテキストにテナント B の StateChange を渡す
        tenant_id: tenant_b,
        table_class: TableClass::TenantScoped,
        payload: serde_json::json!({"cross_tenant_attempt": true}),
        version: 1,
    };

    // contract: verify_tenant_id が TenantIdMismatch エラーを返すこと（DB に到達しない）
    let result = writer.verify_tenant_id(&cross_change);
    assert!(
        result.is_err(),
        "ct: cross-tenant StateChange must be rejected by verify_tenant_id"
    );
}
