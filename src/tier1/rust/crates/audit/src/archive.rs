// 本ファイルは Audit ログのアーカイブ層（MinIO + Object Lock）の抽象。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/10_Audit_Pii_API.md
//     - FR-T1-AUDIT-003（1 年経過後の MinIO アーカイブ、7 年経過後の auto-delete）
//   docs/02_構想設計/adr/ADR-DATA-003-minio.md
//     - MinIO + Object Lock を WORM ストレージとして採用
//
// 役割:
//   warm 層（PostgreSQL）から退避された AuditEntry を不変オブジェクトとして格納し、
//   検索・削除のための最小 API を提供する。production の MinIO 実装は S3 互換 API を
//   `aws-sdk-s3` 経由で叩くが、本リリースの dev / CI 経路は in-memory sink で
//   round-trip と retention 動作を検証する。
//
// オブジェクト命名規約:
//   `audit/<tenant_id>/<YYYY>/<MM>/<DD>/<audit_id>.json`
//   - tenant 単位で prefix 化することで MinIO 側で IAM policy を tenant 別にかけられる
//   - 日付パーティションで MinIO の List 性能を改善
//   - audit_id をオブジェクト名に含めることで重複 PUT を冪等化
//
// Object Lock セマンティクス:
//   production の S3/MinIO 側では PUT 時に `x-amz-object-lock-mode=COMPLIANCE`
//   `x-amz-object-lock-retain-until-date=<entry_timestamp + 7 年>` を指定する。
//   本 trait レイヤでは「不変であることを保証する」契約だけを表現し、
//   実装側で具体的なヘッダ付与・期限設定を担う。

use std::collections::BTreeMap;
use std::sync::RwLock;

use crate::store::{AuditEntry, StoreError};

/// `ArchiveObjectKey` はアーカイブ層オブジェクトの論理キー。
/// 命名規約は本ファイル冒頭を参照。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ArchiveObjectKey {
    /// テナント識別子。
    pub tenant_id: String,
    /// 年（4 桁）。
    pub year: u16,
    /// 月（1〜12）。
    pub month: u8,
    /// 日（1〜31）。
    pub day: u8,
    /// audit_id（hash chain 結果、URL-safe base64）。
    pub audit_id: String,
}

impl ArchiveObjectKey {
    /// S3/MinIO 用のオブジェクトキー文字列に整形する。
    pub fn to_object_path(&self) -> String {
        format!(
            "audit/{}/{:04}/{:02}/{:02}/{}.json",
            self.tenant_id, self.year, self.month, self.day, self.audit_id
        )
    }
}

/// `ArchiveSink` はアーカイブ層への put / list / delete 抽象。
///
/// production: S3/MinIO + Object Lock
/// dev / CI:   InMemoryArchiveSink（process 内 BTreeMap）
pub trait ArchiveSink: Send + Sync {
    /// 1 件 put する。冪等性: 同 key の重複 put は no-op（先勝ち、Object Lock の既定挙動）。
    fn put(&self, key: &ArchiveObjectKey, payload: &[u8]) -> Result<(), StoreError>;
    /// `tenant_id` で絞った全 key を昇順で返す（小規模 dev 用途）。
    /// production では prefix listing で万件超を扱うため、callable は最大件数指定可能。
    fn list_for_tenant(&self, tenant_id: &str) -> Result<Vec<ArchiveObjectKey>, StoreError>;
    /// 削除（FR-T1-AUDIT-003: 7 年経過後の auto-delete）。
    /// production の Object Lock COMPLIANCE モードでは保持期限内の削除は拒否される。
    /// 本 trait は「保持期限を過ぎている前提」での削除契約を表現する。
    fn delete(&self, key: &ArchiveObjectKey) -> Result<(), StoreError>;
    /// 1 件読む（dev 用 round-trip 試験で使う）。production では監査人 audit-export
    /// 経路で使う想定。
    fn get(&self, key: &ArchiveObjectKey) -> Result<Vec<u8>, StoreError>;
}

/// `InMemoryArchiveSink` は process 内 BTreeMap で `ArchiveSink` を満たす実装。
/// dev / CI / 単体テスト向け。production 経路では S3 sink と差し替える。
pub struct InMemoryArchiveSink {
    /// key → payload の map（BTreeMap で list 順を deterministic に）。
    objects: RwLock<BTreeMap<ArchiveObjectKey, Vec<u8>>>,
}

impl InMemoryArchiveSink {
    pub fn new() -> Self {
        Self {
            objects: RwLock::new(BTreeMap::new()),
        }
    }

    /// 件数（テスト用）。
    pub fn len(&self) -> usize {
        self.objects.read().map(|m| m.len()).unwrap_or(0)
    }

    /// 空判定（テスト用）。
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for InMemoryArchiveSink {
    fn default() -> Self {
        Self::new()
    }
}

impl ArchiveSink for InMemoryArchiveSink {
    fn put(&self, key: &ArchiveObjectKey, payload: &[u8]) -> Result<(), StoreError> {
        let mut map = self
            .objects
            .write()
            .map_err(|_| StoreError::LockPoisoned)?;
        // 既存キーは Object Lock 同等で no-op（先勝ち）。
        map.entry(key.clone()).or_insert_with(|| payload.to_vec());
        Ok(())
    }

    fn list_for_tenant(&self, tenant_id: &str) -> Result<Vec<ArchiveObjectKey>, StoreError> {
        let map = self.objects.read().map_err(|_| StoreError::LockPoisoned)?;
        Ok(map
            .keys()
            .filter(|k| k.tenant_id == tenant_id)
            .cloned()
            .collect())
    }

    fn delete(&self, key: &ArchiveObjectKey) -> Result<(), StoreError> {
        let mut map = self
            .objects
            .write()
            .map_err(|_| StoreError::LockPoisoned)?;
        map.remove(key);
        Ok(())
    }

    fn get(&self, key: &ArchiveObjectKey) -> Result<Vec<u8>, StoreError> {
        let map = self.objects.read().map_err(|_| StoreError::LockPoisoned)?;
        map.get(key)
            .cloned()
            .ok_or_else(|| StoreError::Backend("archive object not found".into()))
    }
}

/// `entry_to_object_key` は AuditEntry から ArchiveObjectKey を導出する。
/// timestamp_ms から UTC の年月日を取り出す。
pub fn entry_to_object_key(entry: &AuditEntry) -> ArchiveObjectKey {
    let (year, month, day) = ymd_utc_from_ms(entry.timestamp_ms);
    ArchiveObjectKey {
        tenant_id: entry.tenant_id.clone(),
        year,
        month,
        day,
        audit_id: entry.audit_id.clone(),
    }
}

/// `entry_to_payload` は AuditEntry を canonical JSON bytes に直列化する。
/// store::canonical_bytes と異なり tenant_id / audit_id / prev_id を含む完全な entry を吐く。
pub fn entry_to_payload(entry: &AuditEntry) -> Result<Vec<u8>, StoreError> {
    serde_json::to_vec(entry).map_err(|e| StoreError::Serialize(e.to_string()))
}

/// `ymd_utc_from_ms` は unix milliseconds から UTC 年月日 (Y, M, D) を返す。
///
/// time crate を引き入れずに済むよう、Howard Hinnant の civil_from_days アルゴリズムを
/// 使った proleptic Gregorian 変換を埋め込む（公知のアルゴリズム、proven correct）。
/// 1 ms 単位の精度で 0001-01-01 〜 9999-12-31 範囲を扱える。
pub fn ymd_utc_from_ms(ms: i64) -> (u16, u8, u8) {
    let secs = ms.div_euclid(1000);
    let mut days = secs.div_euclid(86_400);
    // unix epoch (1970-01-01) から civil_from_days 用の 0000-03-01 起点までのオフセット。
    days += 719_468; // 1970-01-01 in Howard Hinnant's days_from_civil
    let era = if days >= 0 { days } else { days - 146_096 } / 146_097;
    let doe = (days - era * 146_097) as u32; // [0, 146_096]
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = doy - (153 * mp + 2) / 5 + 1; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 }; // [1, 12]
    let year = y + if m <= 2 { 1 } else { 0 };
    (year as u16, m as u8, d as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(tid: &str, ts_ms: i64, aid: &str) -> AuditEntry {
        AuditEntry {
            timestamp_ms: ts_ms,
            actor: "u".into(),
            action: "READ".into(),
            resource: "r".into(),
            outcome: "SUCCESS".into(),
            attributes: BTreeMap::new(),
            tenant_id: tid.into(),
            prev_id: "GENESIS".into(),
            audit_id: aid.into(),
        }
    }

    #[test]
    fn ymd_unix_epoch_is_1970_01_01() {
        assert_eq!(ymd_utc_from_ms(0), (1970, 1, 1));
    }

    #[test]
    fn ymd_around_known_dates() {
        // 2023-01-01 00:00:00 UTC = 1672531200000 ms
        assert_eq!(ymd_utc_from_ms(1_672_531_200_000), (2023, 1, 1));
        // 2026-05-01 00:00:00 UTC = 1777593600000 ms
        assert_eq!(ymd_utc_from_ms(1_777_593_600_000), (2026, 5, 1));
        // 2000-02-29 (leap year)
        assert_eq!(ymd_utc_from_ms(951_782_400_000), (2000, 2, 29));
    }

    #[test]
    fn entry_to_object_key_uses_tenant_and_date() {
        let e = entry("tenantA", 1_672_531_200_000, "abc123");
        let key = entry_to_object_key(&e);
        assert_eq!(key.tenant_id, "tenantA");
        assert_eq!(key.year, 2023);
        assert_eq!(key.month, 1);
        assert_eq!(key.day, 1);
        assert_eq!(key.audit_id, "abc123");
        assert_eq!(
            key.to_object_path(),
            "audit/tenantA/2023/01/01/abc123.json"
        );
    }

    #[test]
    fn inmemory_sink_round_trip() {
        let sink = InMemoryArchiveSink::new();
        let key = ArchiveObjectKey {
            tenant_id: "T".into(),
            year: 2024,
            month: 3,
            day: 15,
            audit_id: "abc".into(),
        };
        sink.put(&key, b"payload-1").unwrap();
        assert_eq!(sink.len(), 1);
        let got = sink.get(&key).unwrap();
        assert_eq!(got, b"payload-1");
        // 重複 put は先勝ち（Object Lock 互換）。
        sink.put(&key, b"payload-2").unwrap();
        assert_eq!(sink.get(&key).unwrap(), b"payload-1");
        // delete で消える。
        sink.delete(&key).unwrap();
        assert!(sink.is_empty());
    }

    #[test]
    fn list_for_tenant_filters_correctly() {
        let sink = InMemoryArchiveSink::new();
        sink.put(
            &ArchiveObjectKey {
                tenant_id: "A".into(),
                year: 2024,
                month: 1,
                day: 1,
                audit_id: "1".into(),
            },
            b"p",
        )
        .unwrap();
        sink.put(
            &ArchiveObjectKey {
                tenant_id: "B".into(),
                year: 2024,
                month: 1,
                day: 1,
                audit_id: "2".into(),
            },
            b"p",
        )
        .unwrap();
        sink.put(
            &ArchiveObjectKey {
                tenant_id: "A".into(),
                year: 2024,
                month: 1,
                day: 2,
                audit_id: "3".into(),
            },
            b"p",
        )
        .unwrap();
        let list_a = sink.list_for_tenant("A").unwrap();
        assert_eq!(list_a.len(), 2);
        let list_b = sink.list_for_tenant("B").unwrap();
        assert_eq!(list_b.len(), 1);
    }
}
