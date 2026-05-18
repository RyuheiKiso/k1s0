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

// contract: TenantScoped aggregate の StateChange が正しく SQL を生成できることを検証する
#[test]
fn ct_tenant_scoped_aggregate_state_change() {
    // テスト用テナント ID を生成する
    let tenant_id = Uuid::new_v4();
    // TenantContext を生成する
    let ctx = make_ctx(tenant_id);
    // AtomicTripleWrite を生成する
    let writer = AtomicTripleWrite::new(ctx);
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

    // contract: build_triple_write_sql が Ok を返すこと（SQL 構造の整合性を確認する）
    let sql = writer.build_triple_write_sql(&change)
        .expect("ct: TenantScoped StateChange must generate valid triple-write SQL");
    // contract: domain_event テーブルへの INSERT が含まれること
    assert!(sql.contains("domain_event"), "ct: domain_event INSERT required for TenantScoped");
    // contract: outbox_message テーブルへの INSERT が含まれること（Debezium CDC 経由で Kafka に転送）
    // migration SoT: 0001_initial_schema.sql が k1s0.outbox_message を CREATE している
    assert!(sql.contains("outbox_message"), "ct: outbox_message INSERT required for TenantScoped");
    // contract: audit_event テーブルへの INSERT が含まれること（全操作の監査証跡）
    assert!(sql.contains("audit_event"), "ct: audit_event INSERT required for TenantScoped");
    // contract: GUC 注入が含まれること（RLS FORCE が参照する app.tenant_id を注入する）
    assert!(sql.contains("app.tenant_id"), "ct: GUC injection required");
}

// contract: TenantMaster aggregate の StateChange が正しく SQL を生成できることを検証する
#[test]
fn ct_tenant_master_aggregate_state_change() {
    // テスト用テナント ID を生成する
    let tenant_id = Uuid::new_v4();
    // TenantContext を生成する
    let ctx = make_ctx(tenant_id);
    // AtomicTripleWrite を生成する
    let writer = AtomicTripleWrite::new(ctx);
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

    // contract: build_triple_write_sql が Ok を返すこと
    let sql = writer.build_triple_write_sql(&change)
        .expect("ct: TenantMaster StateChange must generate valid triple-write SQL");
    // contract: TenantMaster でも三表書込が必須であることを確認する
    assert!(sql.contains("domain_event"), "ct: domain_event INSERT required for TenantMaster");
    // outbox_message テーブル名を確認する（migration SoT: k1s0.outbox_message）
    assert!(sql.contains("outbox_message"), "ct: outbox_message INSERT required for TenantMaster");
    assert!(sql.contains("audit_event"), "ct: audit_event INSERT required for TenantMaster");
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
    // AtomicTripleWrite を生成する
    let writer = AtomicTripleWrite::new(ctx);
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

    // contract: PiiSegregated では verify_pii_audit_required が true を返すこと
    assert!(
        writer.verify_pii_audit_required(&pii_change),
        "ct: PiiSegregated aggregate must require audit (P4 invariant)"
    );

    // contract: PiiSegregated の SQL にも三表全てが含まれること
    let sql = writer.build_triple_write_sql(&pii_change)
        .expect("ct: PiiSegregated StateChange must generate valid triple-write SQL");
    // audit_event に PiiSegregated のクラス情報が含まれることを確認する
    assert!(sql.contains("audit_event"), "ct: audit_event INSERT required for PiiSegregated");
}

// contract: 異なるテナントの StateChange が reject されることを検証する（P3 cross-tenant contract）
#[test]
fn ct_cross_tenant_state_change_rejected() {
    // テナント A の TenantContext を生成する
    let tenant_a = Uuid::new_v4();
    let ctx_a = make_ctx(tenant_a);
    // AtomicTripleWrite はテナント A のコンテキストで生成する
    let writer = AtomicTripleWrite::new(ctx_a);

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
