// 本ファイルは archival.rs / retention.rs / archive.rs の統合テスト。
//
// 観点（FR-T1-AUDIT-003）:
//   - 365 日経過した warm entry が ColdMinio (sink) に移送される
//   - 移送後、warm 層から削除される（hash chain は数学的に有効、再 verify はしない運用）
//   - 7 年経過した cold entry が sink から削除される
//   - 削除操作が新規 audit event として記録される（"Audit.Retention.Expire"）
//   - 365 日未満の entry は warm に残る

use crate::archival::*;
use crate::archive::{ArchiveSink, InMemoryArchiveSink};
use crate::retention::{RetentionPolicy, RetentionTier};
use crate::store::{AppendInput, AuditStore, InMemoryAuditStore, QueryInput};
use std::sync::Arc;

/// AuditStoreDeleter blanket impl は AuditStore::delete_warm を呼ぶ adapter。
/// in-memory / postgres 両 store がこれを満たす。
impl<S: AuditStore + ?Sized> AuditStoreDeleter for S {
    fn delete(&self, tenant_id: &str, audit_id: &str) -> Result<(), crate::store::StoreError> {
        self.delete_warm(tenant_id, audit_id)
    }
}

/// テスト用ヘルパ: 指定 timestamp で 1 件 append。
fn append_at(store: &InMemoryAuditStore, tenant: &str, ts_ms: i64, actor: &str) -> String {
    let mut attrs = std::collections::BTreeMap::new();
    attrs.insert("seq".into(), actor.to_string());
    store
        .append(AppendInput {
            timestamp_ms: ts_ms,
            actor: actor.into(),
            action: "READ".into(),
            resource: "r".into(),
            outcome: "SUCCESS".into(),
            attributes: attrs,
            tenant_id: tenant.into(),
        })
        .unwrap()
}

#[test]
fn warm_to_cold_archives_old_entries_only() {
    let store = Arc::new(InMemoryAuditStore::new());
    let sink = Arc::new(InMemoryArchiveSink::new());
    let policy = RetentionPolicy::default();
    let now = 10_000_000_000_000_i64; // 任意の時刻
    // 400 日前（365 日超え → 移行対象）。
    let id_old = append_at(&store, "T", now - 400 * RetentionPolicy::ONE_DAY_MS, "old");
    // 30 日前（HotLoki → 残す）。
    let id_recent = append_at(&store, "T", now - 30 * RetentionPolicy::ONE_DAY_MS, "recent");

    let runner = RetentionRunner {
        store: store.clone(),
        sink: sink.clone(),
        policy,
        max_per_tier: 0,
    };
    let stats = runner
        .run_once(store.as_ref(), &["T".into()], now)
        .unwrap();
    assert_eq!(stats.warm_to_cold_archived, 1, "1 entry archived");
    assert_eq!(stats.warm_to_cold_deleted, 1, "1 entry deleted from warm");
    assert_eq!(stats.cold_to_expired_deleted, 0);
    // 古い方は warm から消えている。
    let remaining = store
        .query(QueryInput {
            tenant_id: "T".into(),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].audit_id, id_recent);
    // sink には古い方が残っている。
    let listed = sink.list_for_tenant("T").unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].audit_id, id_old);
}

#[test]
fn cold_to_expired_deletes_after_seven_years_and_audits() {
    let store = Arc::new(InMemoryAuditStore::new());
    let sink = Arc::new(InMemoryArchiveSink::new());
    let policy = RetentionPolicy::default();
    let now = 10_000_000_000_000_i64;
    // 8 年前 entry を sink に直接 put（warm 経由を skip して expire 経路だけテスト）。
    let eight_years_ago = now - 8 * 365 * RetentionPolicy::ONE_DAY_MS;
    let key = crate::archive::ArchiveObjectKey {
        tenant_id: "T".into(),
        // ymd を 8 年前から計算する（archive::ymd_utc_from_ms を使用）。
        ..{
            let (y, m, d) = crate::archive::ymd_utc_from_ms(eight_years_ago);
            crate::archive::ArchiveObjectKey {
                tenant_id: String::new(),
                year: y,
                month: m,
                day: d,
                audit_id: "ancient-id".into(),
            }
        }
    };
    sink.put(&key, b"{}").unwrap();

    let runner = RetentionRunner {
        store: store.clone(),
        sink: sink.clone(),
        policy,
        max_per_tier: 0,
    };
    let stats = runner
        .run_once(store.as_ref(), &["T".into()], now)
        .unwrap();
    assert_eq!(stats.cold_to_expired_deleted, 1, "8 年前 entry が削除される");
    assert_eq!(
        stats.expired_audit_emitted, 1,
        "削除操作が audit に記録される"
    );
    // sink から消えている。
    assert!(sink.is_empty(), "sink should be empty after expire");
    // store に新規 audit event（Audit.Retention.Expire）が積まれている。
    let events = store
        .query(QueryInput {
            tenant_id: "T".into(),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].action, "Audit.Retention.Expire");
    assert_eq!(events[0].actor, "system:retention-runner");
    assert!(events[0].resource.contains("audit-archive:T:ancient-id"));
    assert_eq!(
        events[0].attributes.get("retention_action").unwrap(),
        "expire-cold"
    );
}

#[test]
fn cold_to_expired_skips_recent_archive_entries() {
    let store = Arc::new(InMemoryAuditStore::new());
    let sink = Arc::new(InMemoryArchiveSink::new());
    let policy = RetentionPolicy::default();
    let now = 10_000_000_000_000_i64;
    // 2 年前の archive entry（cold 層内、まだ削除しない）。
    let two_years_ago = now - 2 * 365 * RetentionPolicy::ONE_DAY_MS;
    let (y, m, d) = crate::archive::ymd_utc_from_ms(two_years_ago);
    let key = crate::archive::ArchiveObjectKey {
        tenant_id: "T".into(),
        year: y,
        month: m,
        day: d,
        audit_id: "still-cold".into(),
    };
    sink.put(&key, b"{}").unwrap();
    let runner = RetentionRunner {
        store: store.clone(),
        sink: sink.clone(),
        policy,
        max_per_tier: 0,
    };
    let stats = runner
        .run_once(store.as_ref(), &["T".into()], now)
        .unwrap();
    assert_eq!(stats.cold_to_expired_deleted, 0);
    assert_eq!(sink.len(), 1, "still-cold entry must remain in sink");
}

#[test]
fn max_per_tier_caps_processing() {
    let store = Arc::new(InMemoryAuditStore::new());
    let sink = Arc::new(InMemoryArchiveSink::new());
    let policy = RetentionPolicy::default();
    let now = 10_000_000_000_000_i64;
    // 5 件すべて 400 日前。
    for i in 0..5 {
        append_at(&store, "T", now - 400 * RetentionPolicy::ONE_DAY_MS - i, &format!("e{}", i));
    }
    let runner = RetentionRunner {
        store: store.clone(),
        sink: sink.clone(),
        policy,
        max_per_tier: 2,
    };
    let stats = runner
        .run_once(store.as_ref(), &["T".into()], now)
        .unwrap();
    assert_eq!(stats.warm_to_cold_archived, 2);
    assert_eq!(stats.warm_to_cold_deleted, 2);
    // warm に 3 件残る、次回 run で続きを処理。
    let remaining = store
        .query(QueryInput {
            tenant_id: "T".into(),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(remaining.len(), 3);
}

#[test]
fn ymd_to_ms_round_trip() {
    // archive::ymd_utc_from_ms と archival::ymd_to_ms の往復一致を確認。
    let cases = [
        (1970_u16, 1_u8, 1_u8),
        (2000, 2, 29),
        (2023, 1, 1),
        (2026, 5, 1),
    ];
    for (y, m, d) in cases {
        let ms = crate::archival::ymd_to_ms(y, m, d);
        let back = crate::archive::ymd_utc_from_ms(ms);
        assert_eq!(back, (y, m, d), "round-trip {} {} {}", y, m, d);
    }
}

#[test]
fn tier_helper_categorizes_correctly() {
    // 単体動作確認: retention.rs の判定 helper が runner と整合する。
    let p = RetentionPolicy::default();
    assert_eq!(p.tier_for_age_ms(0), RetentionTier::HotLoki);
    assert_eq!(
        p.tier_for_age_ms(100 * RetentionPolicy::ONE_DAY_MS),
        RetentionTier::WarmPg
    );
    assert_eq!(
        p.tier_for_age_ms(400 * RetentionPolicy::ONE_DAY_MS),
        RetentionTier::ColdMinio
    );
    assert_eq!(
        p.tier_for_age_ms(3000 * RetentionPolicy::ONE_DAY_MS),
        RetentionTier::Expired
    );
}
