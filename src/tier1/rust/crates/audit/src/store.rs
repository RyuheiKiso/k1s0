// 本ファイルは Audit WORM ストアの中核実装。
//
// 設計正典:
//   docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/01_tier1全体コンポーネント俯瞰.md
//     - DS-SW-COMP-007（t1-audit Pod、WORM 追記専用）
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/10_Audit_Pii_API.md
//
// データモデル:
//   各 audit event は AppendEntry として保持され、追記専用（更新・削除なし）。
//   audit_id は前のエントリの audit_id（または "GENESIS"）と canonical event JSON を
//   連結したものを SHA-256 で hash したものの hex 表現。
//
// ハッシュチェーン強度:
//   過去のイベントの 1 byte を改ざんしても、それ以降の audit_id がすべて変わるため
//   末尾エントリの audit_id を独立に保管しておけば改ざんを検知できる
//   （NFR-H-INT-001 / 002 完整性要件と整合）。
//
// 永続化:
//   本リリースは in-memory backend のみ。Postgres backend は同 trait を満たす
//   別実装として `crate::store::pg` (将来追加) で提供予定。trait を介して
//   handler から swap 可能にする。

use std::sync::RwLock;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// AuditStore のエラー型。
/// thiserror crate は workspace dep に未追加のため Display / Error を手書きで実装する。
#[derive(Debug)]
pub enum StoreError {
    /// 保存中の io / シリアライズエラー。
    Serialize(String),
    /// 内部ロック獲得失敗（poisoned）。
    LockPoisoned,
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreError::Serialize(s) => write!(f, "serialize: {}", s),
            StoreError::LockPoisoned => write!(f, "lock poisoned"),
        }
    }
}

impl std::error::Error for StoreError {}

/// 保存される event 構造（proto AuditEvent と等価だが、Rust 側で hash 計算に使う canonical 表現）。
///
/// timestamp は unix milliseconds で表現する（NFR-A-SLA-* に整合）。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuditEntry {
    /// 発生時刻（unix milliseconds）。
    pub timestamp_ms: i64,
    /// 操作主体。
    pub actor: String,
    /// 操作種別。
    pub action: String,
    /// 対象リソース URN。
    pub resource: String,
    /// 操作結果。
    pub outcome: String,
    /// 追加 attributes。
    pub attributes: std::collections::BTreeMap<String, String>,
    /// テナント識別子（本層で append 時に固定保存し、querying のテナント境界を担保）。
    pub tenant_id: String,
    /// チェーン上の前 entry の audit_id（ルートは "GENESIS"）。
    pub prev_id: String,
    /// 自 entry の audit_id（hash chain 計算結果）。
    pub audit_id: String,
}

/// AuditStore は WORM ストアの操作集合。
pub trait AuditStore: Send + Sync {
    /// 単一 event を append する。返却の audit_id は SHA-256 hash chain 結果。
    fn append(&self, entry: AppendInput) -> Result<String, StoreError>;
    /// 範囲 + filter で query する。timestamp 昇順で返す。
    fn query(&self, query: QueryInput) -> Result<Vec<AuditEntry>, StoreError>;
    /// チェーン整合性検証（全 entry の audit_id を再計算して一致確認）。
    fn verify_chain(&self) -> Result<(), StoreError>;
    /// FR-T1-AUDIT-002 の VerifyChain RPC 用詳細検証。
    /// 全グローバルチェーンを末尾まで歩き、prev_id / audit_id の整合性を確認する。
    /// 改ざん検知時は first_bad_sequence（1-based、グローバル順序）と reason を返す。
    /// `tenant_id` / `from_ms` / `to_ms` は checked_count の範囲絞り込みのみに使う
    /// （チェーン検証自体はテナント横断のグローバル走査）。
    fn verify_chain_detail(
        &self,
        tenant_id: &str,
        from_ms: Option<i64>,
        to_ms: Option<i64>,
    ) -> Result<VerifyOutcome, StoreError>;
}

/// VerifyChain の詳細検証結果。proto VerifyChainResponse と意味的対応。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifyOutcome {
    /// チェーン全体が整合していれば true。
    pub valid: bool,
    /// テナント / 時刻範囲フィルタ後の検証対象件数。
    pub checked_count: i64,
    /// 不整合検出時、最初に失敗した entry の 1-based グローバル順序番号。valid 時は 0。
    pub first_bad_sequence: i64,
    /// 不整合検出時の理由。valid 時は空文字。
    pub reason: String,
}

/// append() に渡す入力（audit_id / prev_id は内部計算）。
#[derive(Debug, Clone)]
pub struct AppendInput {
    pub timestamp_ms: i64,
    pub actor: String,
    pub action: String,
    pub resource: String,
    pub outcome: String,
    pub attributes: std::collections::BTreeMap<String, String>,
    pub tenant_id: String,
}

/// query() に渡す入力（範囲 + filter + limit、tenant_id は必ず指定）。
#[derive(Debug, Clone, Default)]
pub struct QueryInput {
    pub from_ms: Option<i64>,
    pub to_ms: Option<i64>,
    pub filters: std::collections::BTreeMap<String, String>,
    pub limit: usize,
    pub tenant_id: String,
}

/// in-memory ハッシュチェーン実装。
///
/// 内部は RwLock<Vec<AuditEntry>>（append は短時間 Write、query は Read）。
/// プロセス再起動で消えるため、production では同 trait の Postgres 実装に
/// swap する想定。テストでは本実装で full lifecycle をテストできる。
pub struct InMemoryAuditStore {
    entries: RwLock<Vec<AuditEntry>>,
    /// 最後に append された entry の audit_id を保持（次の append の prev_id）。
    last_id: RwLock<String>,
}

impl InMemoryAuditStore {
    /// 新規空 store を生成。最初の prev_id は "GENESIS"。
    pub fn new() -> Self {
        InMemoryAuditStore {
            entries: RwLock::new(Vec::new()),
            last_id: RwLock::new("GENESIS".to_string()),
        }
    }
}

impl Default for InMemoryAuditStore {
    fn default() -> Self {
        Self::new()
    }
}

/// canonical_bytes は AppendInput を deterministic な JSON で serialize した
/// bytes を返す。BTreeMap でキー順を固定し、tenant_id / actor / action 等の順序も固定する。
fn canonical_bytes(input: &AppendInput) -> Result<Vec<u8>, StoreError> {
    // serde_json は struct field 順を struct 定義順で出力するため、struct を
    // 1 件作って serialize すれば deterministic。BTreeMap 経由で attributes も整列する。
    #[derive(Serialize)]
    struct Canon<'a> {
        timestamp_ms: i64,
        actor: &'a str,
        action: &'a str,
        resource: &'a str,
        outcome: &'a str,
        attributes: &'a std::collections::BTreeMap<String, String>,
        tenant_id: &'a str,
    }
    let canon = Canon {
        timestamp_ms: input.timestamp_ms,
        actor: &input.actor,
        action: &input.action,
        resource: &input.resource,
        outcome: &input.outcome,
        attributes: &input.attributes,
        tenant_id: &input.tenant_id,
    };
    serde_json::to_vec(&canon).map_err(|e| StoreError::Serialize(e.to_string()))
}

/// hash_chain は prev_id + canonical_bytes から SHA-256 を計算し hex で返す。
fn hash_chain(prev_id: &str, canon: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(prev_id.as_bytes());
    h.update(canon);
    hex::encode(h.finalize())
}

impl AuditStore for InMemoryAuditStore {
    fn append(&self, input: AppendInput) -> Result<String, StoreError> {
        // canonical bytes を計算。
        let canon = canonical_bytes(&input)?;
        // 直前の audit_id を読む（再起動後も "GENESIS" から始まる）。
        let prev_id = self
            .last_id
            .read()
            .map_err(|_| StoreError::LockPoisoned)?
            .clone();
        // 新 audit_id を計算。
        let audit_id = hash_chain(&prev_id, &canon);
        // entry を構築して追記。
        let entry = AuditEntry {
            timestamp_ms: input.timestamp_ms,
            actor: input.actor,
            action: input.action,
            resource: input.resource,
            outcome: input.outcome,
            attributes: input.attributes,
            tenant_id: input.tenant_id,
            prev_id,
            audit_id: audit_id.clone(),
        };
        // 排他で append + last_id 更新。
        let mut entries = self
            .entries
            .write()
            .map_err(|_| StoreError::LockPoisoned)?;
        entries.push(entry);
        let mut last = self.last_id.write().map_err(|_| StoreError::LockPoisoned)?;
        *last = audit_id.clone();
        Ok(audit_id)
    }

    fn query(&self, q: QueryInput) -> Result<Vec<AuditEntry>, StoreError> {
        let entries = self.entries.read().map_err(|_| StoreError::LockPoisoned)?;
        // limit 既定値。proto 仕様に従い 0 を 100 として扱う。
        let limit = if q.limit == 0 { 100 } else { q.limit.min(1000) };
        let mut out: Vec<AuditEntry> = entries
            .iter()
            .filter(|e| !q.tenant_id.is_empty() && e.tenant_id == q.tenant_id)
            .filter(|e| q.from_ms.map_or(true, |f| e.timestamp_ms >= f))
            .filter(|e| q.to_ms.map_or(true, |t| e.timestamp_ms <= t))
            .filter(|e| {
                q.filters
                    .iter()
                    .all(|(k, v)| match e.attributes.get(k) {
                        Some(av) => av == v,
                        None => false,
                    })
            })
            .cloned()
            .collect();
        // timestamp 昇順 sort。
        out.sort_by_key(|e| e.timestamp_ms);
        out.truncate(limit);
        Ok(out)
    }

    fn verify_chain(&self) -> Result<(), StoreError> {
        let outcome = self.verify_chain_detail("", None, None)?;
        if outcome.valid {
            Ok(())
        } else {
            Err(StoreError::Serialize(format!(
                "verify_chain failed at sequence {}: {}",
                outcome.first_bad_sequence, outcome.reason
            )))
        }
    }

    fn verify_chain_detail(
        &self,
        tenant_id: &str,
        from_ms: Option<i64>,
        to_ms: Option<i64>,
    ) -> Result<VerifyOutcome, StoreError> {
        let entries = self.entries.read().map_err(|_| StoreError::LockPoisoned)?;
        let mut prev = "GENESIS".to_string();
        let mut checked: i64 = 0;
        for (idx, e) in entries.iter().enumerate() {
            if e.prev_id != prev {
                return Ok(VerifyOutcome {
                    valid: false,
                    checked_count: checked,
                    first_bad_sequence: (idx as i64) + 1,
                    reason: format!(
                        "prev_id mismatch at sequence {}: expected {}, got {}",
                        idx + 1,
                        prev,
                        e.prev_id
                    ),
                });
            }
            let input = AppendInput {
                timestamp_ms: e.timestamp_ms,
                actor: e.actor.clone(),
                action: e.action.clone(),
                resource: e.resource.clone(),
                outcome: e.outcome.clone(),
                attributes: e.attributes.clone(),
                tenant_id: e.tenant_id.clone(),
            };
            let canon = canonical_bytes(&input)?;
            let computed = hash_chain(&prev, &canon);
            if computed != e.audit_id {
                return Ok(VerifyOutcome {
                    valid: false,
                    checked_count: checked,
                    first_bad_sequence: (idx as i64) + 1,
                    reason: format!(
                        "audit_id mismatch at sequence {}: expected {}, got {}",
                        idx + 1,
                        e.audit_id,
                        computed
                    ),
                });
            }
            // 範囲内ならカウント。tenant_id="" はテナント無視（全件カウント）。
            let in_tenant = tenant_id.is_empty() || e.tenant_id == tenant_id;
            let in_range = from_ms.is_none_or(|f| e.timestamp_ms >= f)
                && to_ms.is_none_or(|t| e.timestamp_ms <= t);
            if in_tenant && in_range {
                checked += 1;
            }
            prev = e.audit_id.clone();
        }
        Ok(VerifyOutcome {
            valid: true,
            checked_count: checked,
            first_bad_sequence: 0,
            reason: String::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_input(ts: i64, actor: &str, tenant: &str) -> AppendInput {
        AppendInput {
            timestamp_ms: ts,
            actor: actor.to_string(),
            action: "READ".to_string(),
            resource: "k1s0:tenant:T:resource:secret/db".to_string(),
            outcome: "SUCCESS".to_string(),
            attributes: Default::default(),
            tenant_id: tenant.to_string(),
        }
    }

    #[test]
    fn append_returns_audit_id_and_chains() {
        let s = InMemoryAuditStore::new();
        let id1 = s.append(make_input(1, "u1", "T")).unwrap();
        let id2 = s.append(make_input(2, "u2", "T")).unwrap();
        // 異なる id が出る。
        assert_ne!(id1, id2);
        // 第 2 entry の prev_id は第 1 の audit_id。
        let entries = s.entries.read().unwrap();
        assert_eq!(entries[1].prev_id, id1);
    }

    #[test]
    fn append_deterministic_hash() {
        // 同一入力で同一 hash が出る（ただし prev_id が "GENESIS" の最初のみ）。
        let a = InMemoryAuditStore::new();
        let b = InMemoryAuditStore::new();
        let id_a = a.append(make_input(100, "u", "T")).unwrap();
        let id_b = b.append(make_input(100, "u", "T")).unwrap();
        assert_eq!(id_a, id_b);
    }

    #[test]
    fn query_filters_by_tenant() {
        let s = InMemoryAuditStore::new();
        s.append(make_input(1, "u1", "T1")).unwrap();
        s.append(make_input(2, "u2", "T2")).unwrap();
        s.append(make_input(3, "u3", "T1")).unwrap();
        let r = s
            .query(QueryInput {
                tenant_id: "T1".to_string(),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(r.len(), 2);
        assert!(r.iter().all(|e| e.tenant_id == "T1"));
    }

    #[test]
    fn query_filters_by_range_and_limit() {
        let s = InMemoryAuditStore::new();
        for i in 1..=10 {
            s.append(make_input(i, "u", "T")).unwrap();
        }
        let r = s
            .query(QueryInput {
                tenant_id: "T".to_string(),
                from_ms: Some(3),
                to_ms: Some(7),
                limit: 0, // → 100 default
                ..Default::default()
            })
            .unwrap();
        // 範囲 3..=7 の 5 件。
        assert_eq!(r.len(), 5);
        assert_eq!(r[0].timestamp_ms, 3);
        assert_eq!(r[4].timestamp_ms, 7);

        // limit を効かせる。
        let r2 = s
            .query(QueryInput {
                tenant_id: "T".to_string(),
                limit: 3,
                ..Default::default()
            })
            .unwrap();
        assert_eq!(r2.len(), 3);
    }

    #[test]
    fn query_filters_by_attributes() {
        let s = InMemoryAuditStore::new();
        let mut a1 = make_input(1, "u1", "T");
        a1.attributes.insert("ip".into(), "10.0.0.1".into());
        let mut a2 = make_input(2, "u2", "T");
        a2.attributes.insert("ip".into(), "10.0.0.2".into());
        s.append(a1).unwrap();
        s.append(a2).unwrap();
        let r = s
            .query(QueryInput {
                tenant_id: "T".to_string(),
                filters: [("ip".to_string(), "10.0.0.2".to_string())]
                    .into_iter()
                    .collect(),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].timestamp_ms, 2);
    }

    #[test]
    fn verify_chain_passes_after_appends() {
        let s = InMemoryAuditStore::new();
        for i in 1..=5 {
            s.append(make_input(i, "u", "T")).unwrap();
        }
        s.verify_chain().expect("chain valid");
    }

    #[test]
    fn verify_chain_detects_tamper() {
        let s = InMemoryAuditStore::new();
        s.append(make_input(1, "u", "T")).unwrap();
        s.append(make_input(2, "u", "T")).unwrap();
        // 1 件目の actor を直接書き換える（WORM 違反、検知されるべき）。
        {
            let mut entries = s.entries.write().unwrap();
            entries[0].actor = "u-evil".to_string();
        }
        let r = s.verify_chain();
        assert!(r.is_err(), "tamper should be detected");
    }

    /// VerifyChain RPC の詳細応答（proto VerifyChainResponse 互換）が、
    /// 改ざんされた entry の **正確な sequence 番号 + 検知理由** を
    /// 返すことを検証する。NFR-H-INT-001 / 002 の核心要件:
    /// "完全性違反は検知可能 + 違反箇所が特定可能"。
    #[test]
    fn verify_chain_detail_returns_first_bad_sequence_with_reason() {
        let s = InMemoryAuditStore::new();
        s.append(make_input(1, "u1", "T")).unwrap();
        s.append(make_input(2, "u2", "T")).unwrap();
        s.append(make_input(3, "u3", "T")).unwrap();
        // 改ざん前 detail: valid=true / checked=3 / first_bad=0
        let ok = s.verify_chain_detail("T", None, None).unwrap();
        assert!(ok.valid, "before tamper should be valid");
        assert_eq!(ok.checked_count, 3);
        assert_eq!(ok.first_bad_sequence, 0);
        assert_eq!(ok.reason, "");
        // 中央の 2 件目を改ざん。
        {
            let mut entries = s.entries.write().unwrap();
            entries[1].action = "tampered".to_string();
        }
        let bad = s.verify_chain_detail("T", None, None).unwrap();
        assert!(!bad.valid, "tamper must be reported invalid");
        // checked_count は valid だった entry 数（1 件目）まで。
        assert_eq!(
            bad.checked_count, 1,
            "checked_count should stop at the entry preceding the tamper"
        );
        // 2 件目で audit_id 不整合を検知するはず。
        assert_eq!(
            bad.first_bad_sequence, 2,
            "first_bad_sequence must point to the tampered entry"
        );
        assert!(
            bad.reason.contains("audit_id mismatch at sequence 2"),
            "reason should describe the actual mismatch (got: {:?})",
            bad.reason
        );
    }

    /// 中央の entry を **削除** した場合、それ以降の prev_id chain が壊れて
    /// 検知される。InMemory store は entries Vec を露出する WORM 違反テスト用 API
    /// を持たないため、unsafe な write 直接編集を使う。
    #[test]
    fn verify_chain_detail_detects_deletion_via_prev_id_break() {
        let s = InMemoryAuditStore::new();
        s.append(make_input(1, "u1", "T")).unwrap();
        s.append(make_input(2, "u2", "T")).unwrap();
        s.append(make_input(3, "u3", "T")).unwrap();
        // 2 件目を削除（→ 3 件目の prev_id が 1 件目の audit_id を指さない状態）。
        {
            let mut entries = s.entries.write().unwrap();
            entries.remove(1);
        }
        let bad = s.verify_chain_detail("T", None, None).unwrap();
        assert!(!bad.valid, "deletion must be reported invalid");
        // 1 件目はそのまま valid、2 件目の位置（旧 3 件目）で prev_id mismatch。
        assert_eq!(bad.first_bad_sequence, 2);
        assert!(
            bad.reason.contains("prev_id mismatch at sequence 2"),
            "reason should say prev_id mismatch (got: {:?})",
            bad.reason
        );
    }
}
