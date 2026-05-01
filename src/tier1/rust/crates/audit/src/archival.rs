// 本ファイルは Audit ログの retention runner（warm → cold 移行 + cold 削除）。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/10_Audit_Pii_API.md
//     - FR-T1-AUDIT-003 受け入れ基準:
//       * 90 日経過後、自動的に Kafka → PostgreSQL に移行
//       * 1 年経過後、自動的に PostgreSQL → MinIO にエクスポート
//       * 7 年経過後、自動削除（削除操作も Audit ログに記録）
//
// 役割:
//   日次 cron（K8s CronJob 想定）から呼び出される `RetentionRunner::run_once` を提供する。
//   PostgreSQL 上の WarmPg entry のうち > 365 日経過したものを ArchiveSink に export し、
//   Postgres 側から論理削除（あるいは物理 DELETE）する。
//   ArchiveSink 上の Cold entry のうち > 7 年経過したものを delete し、
//   削除操作自身を新規 audit event として記録する（"Audit.Retention.Expire" action）。
//
// 構造:
//   `RetentionRunner` は再入可能（並行 1 インスタンスのみを想定）の async タスクで、
//   バッチサイズ単位で処理する。1 回の run_once で `max_per_tier` 件まで進めて切る
//   （長時間 lock を回避し、後続 audit write を block しない）。
//
// 失敗時挙動:
//   - PostgreSQL 接続失敗: ArchivalError::Backend を返し、運用 alert で検知
//   - ArchiveSink put 失敗: Postgres delete を行わずスキップ（次回 run で再試行）
//   - ArchiveSink delete 失敗: warn ログのみで継続
//   各失敗は別々のメトリクスで監視可能にする想定（reporter 引数を取る形に拡張可能）。

use std::sync::Arc;

use crate::archive::{entry_to_object_key, entry_to_payload, ArchiveSink};
use crate::store::{AppendInput, AuditStore, QueryInput, StoreError};
use crate::retention::RetentionPolicy;

/// `RetentionRunner` は warm → cold → expired の段階的 lifecycle を進めるバッチタスク。
pub struct RetentionRunner {
    /// warm 層の永続ストア（PostgreSQL or in-memory）。
    pub store: Arc<dyn AuditStore>,
    /// cold 層の archive sink（MinIO or in-memory）。
    pub sink: Arc<dyn ArchiveSink>,
    /// retention 閾値（既定 90/365/2555 日）。
    pub policy: RetentionPolicy,
    /// 1 run あたりの最大処理件数（warm→cold / cold→expired それぞれ）。
    /// 0 は「無制限」を意味する。production では 1000〜10000 推奨。
    pub max_per_tier: usize,
}

/// `RetentionStats` は `run_once` の結果統計。
/// 削除件数 / アーカイブ件数 / 失敗件数を返し、運用側はメトリクスとして吸い上げる想定。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RetentionStats {
    /// warm → cold で sink にコピーした件数。
    pub warm_to_cold_archived: usize,
    /// warm → cold 後、warm 層から削除した件数（archived と通常一致）。
    pub warm_to_cold_deleted: usize,
    /// cold → expired で sink から削除した件数。
    pub cold_to_expired_deleted: usize,
    /// 7 年経過削除イベントを audit に記録した件数（cold_to_expired_deleted と通常一致）。
    pub expired_audit_emitted: usize,
    /// 何らかの失敗で次回 run に持ち越した件数（個別 entry 単位）。
    pub failures: usize,
}

/// `ArchivalError` は run_once が致命的に中断するケースのエラー。
/// 個別 entry 失敗は stats.failures にカウントするのみで、ここまでは到達しない。
#[derive(Debug)]
pub enum ArchivalError {
    /// store からの query / append / delete 失敗（致命）。
    Store(StoreError),
}

impl std::fmt::Display for ArchivalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArchivalError::Store(e) => write!(f, "store: {}", e),
        }
    }
}

impl std::error::Error for ArchivalError {}

impl From<StoreError> for ArchivalError {
    fn from(e: StoreError) -> Self {
        ArchivalError::Store(e)
    }
}

/// `AuditStoreDeleter` trait は warm 層から特定 entry を削除する能力。
///
/// 既存 `AuditStore` trait は append-only WORM 設計のため delete を持たない。
/// retention runner は閾値超過 entry を warm 層から消す必要があるため、
/// 本 trait 実装が必要な store のみがアーカイブ移行に参加できる。
/// in-memory / postgres 両 store でこれを別途実装する。
pub trait AuditStoreDeleter: AuditStore {
    /// `audit_id` で entry を warm 層から削除する。
    /// `tenant_id` で二重防御（テナント越境削除を防ぐ）。
    fn delete(&self, tenant_id: &str, audit_id: &str) -> Result<(), StoreError>;
}

/// 全 `AuditStore` 実装に対して `AuditStoreDeleter` を `delete_warm` 経由で提供する blanket impl。
/// `delete_warm` は trait の既定実装で「未対応」を返すため、retention runner は
/// 実装側が override しているケースのみ実際に warm を削除する（既定 store は no-op）。
impl<S: AuditStore + ?Sized> AuditStoreDeleter for S {
    fn delete(&self, tenant_id: &str, audit_id: &str) -> Result<(), StoreError> {
        self.delete_warm(tenant_id, audit_id)
    }
}

impl RetentionRunner {
    /// 単発実行。warm→cold 移行 → cold→expired 削除 → expired audit 発火 を順に行う。
    ///
    /// `tenants` は処理対象テナント集合。production では「全テナント」を query するが、
    /// 本 API では呼出側が明示する（テナント単位のスケジュール多重を許す）。
    /// `now_ms` は時刻注入（テストで決定論的に実行できるよう、運用環境では `chrono::Utc::now`
    /// を渡す）。
    /// `deleter` は warm 層からの delete を行うアダプタ（store と同じインスタンス想定）。
    pub fn run_once<D: AuditStoreDeleter + ?Sized>(
        &self,
        deleter: &D,
        tenants: &[String],
        now_ms: i64,
    ) -> Result<RetentionStats, ArchivalError> {
        let mut stats = RetentionStats::default();
        for tenant in tenants {
            self.archive_warm_to_cold(deleter, tenant, now_ms, &mut stats)?;
            self.expire_cold(deleter, tenant, now_ms, &mut stats)?;
        }
        Ok(stats)
    }

    /// warm 層 (store) のうち > 365 日経過した entry を ArchiveSink に export し、
    /// 成功した分のみ store から delete する。
    fn archive_warm_to_cold<D: AuditStoreDeleter + ?Sized>(
        &self,
        deleter: &D,
        tenant: &str,
        now_ms: i64,
        stats: &mut RetentionStats,
    ) -> Result<(), ArchivalError> {
        let cutoff = self.policy.warm_to_cold_cutoff_ms(now_ms);
        // warm 層の対象 entry を query する。
        let q = QueryInput {
            from_ms: None,
            to_ms: Some(cutoff),
            filters: Default::default(),
            limit: if self.max_per_tier == 0 {
                usize::MAX
            } else {
                self.max_per_tier
            },
            tenant_id: tenant.to_string(),
        };
        let entries = self.store.query(q)?;
        for e in &entries {
            // sink への put → store からの delete を atomic に近い順序で行う
            // （sink put 失敗時は delete しない / 次回 run で再試行）。
            let key = entry_to_object_key(e);
            let payload = match entry_to_payload(e) {
                Ok(p) => p,
                Err(_) => {
                    stats.failures += 1;
                    continue;
                }
            };
            if self.sink.put(&key, &payload).is_err() {
                stats.failures += 1;
                continue;
            }
            stats.warm_to_cold_archived += 1;
            // warm 層から削除する。失敗は次回 run で sink put が冪等（先勝ち）なので再試行可。
            if deleter.delete(&e.tenant_id, &e.audit_id).is_err() {
                stats.failures += 1;
                continue;
            }
            stats.warm_to_cold_deleted += 1;
        }
        Ok(())
    }

    /// cold 層（ArchiveSink）のうち > 7 年経過したオブジェクトを削除し、
    /// 削除操作自身を audit event として store に append する。
    fn expire_cold<D: AuditStoreDeleter + ?Sized>(
        &self,
        _deleter: &D,
        tenant: &str,
        now_ms: i64,
        stats: &mut RetentionStats,
    ) -> Result<(), ArchivalError> {
        let cutoff = self.policy.cold_to_expired_cutoff_ms(now_ms);
        let keys = self.sink.list_for_tenant(tenant).map_err(ArchivalError::Store)?;
        let mut processed = 0usize;
        for key in keys {
            // key の年月日から期限判定する。月初 00:00:00 UTC を採用（保守的）。
            let key_ms = ymd_to_ms(key.year, key.month, key.day);
            if key_ms > cutoff {
                continue; // まだ 7 年経過していない
            }
            // 個別失敗は warn 扱い（stats.failures をインクリメントして継続）。
            if self.sink.delete(&key).is_err() {
                stats.failures += 1;
                continue;
            }
            stats.cold_to_expired_deleted += 1;
            // 削除操作を audit event として記録する（FR-T1-AUDIT-003 受け入れ基準
            // 「7 年経過後、自動削除（削除操作も Audit ログに記録）」）。
            let attributes = expire_audit_attributes(&key);
            let input = AppendInput {
                timestamp_ms: now_ms,
                actor: "system:retention-runner".to_string(),
                action: "Audit.Retention.Expire".to_string(),
                resource: format!(
                    "audit-archive:{}:{}",
                    key.tenant_id, key.audit_id
                ),
                outcome: "SUCCESS".to_string(),
                attributes,
                tenant_id: key.tenant_id.clone(),
            };
            if self.store.append(input).is_ok() {
                stats.expired_audit_emitted += 1;
            } else {
                stats.failures += 1;
            }
            processed += 1;
            if self.max_per_tier > 0 && processed >= self.max_per_tier {
                break;
            }
        }
        Ok(())
    }
}

/// `expire_audit_attributes` は削除イベントの追加 attributes を組み立てる。
fn expire_audit_attributes(key: &crate::archive::ArchiveObjectKey) -> std::collections::BTreeMap<String, String> {
    let mut attrs = std::collections::BTreeMap::new();
    attrs.insert("retention_action".to_string(), "expire-cold".to_string());
    attrs.insert("archive_object".to_string(), key.to_object_path());
    attrs.insert("archived_audit_id".to_string(), key.audit_id.clone());
    attrs.insert("retention_days".to_string(), "2555".to_string());
    attrs
}

/// `ymd_to_ms` は (year, month, day) 00:00:00 UTC を unix milliseconds に変換する。
/// archive.rs の `ymd_utc_from_ms` の逆関数。
pub fn ymd_to_ms(year: u16, month: u8, day: u8) -> i64 {
    // Howard Hinnant's days_from_civil の逆。
    let y = year as i64 - if month <= 2 { 1 } else { 0 };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = (y - era * 400) as u32; // [0, 399]
    let m = month as u32;
    let mp = if m > 2 { m - 3 } else { m + 9 }; // [0, 11]
    let d = day as u32;
    let doy = (153 * mp + 2) / 5 + d - 1; // [0, 365]
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy; // [0, 146_096]
    let days = era * 146_097 + doe as i64 - 719_468; // unix days
    days * 86_400 * 1000
}

#[cfg(test)]
#[path = "archival_tests.rs"]
mod archival_tests;
