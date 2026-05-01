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
//   連結したものを SHA-256 で hash したもの。
//
// ハッシュ表現（FR-T1-AUDIT-001）:
//   要件「ハッシュは SHA-256, base64 エンコード」に従い、URL/ファイル名安全な
//   base64（padding なし、44 文字 → 43 文字）で文字列化する。
//   - 出力: `URL_SAFE_NO_PAD` Engine（'-'/'_' を使う、'='なし、43 文字）
//   - 入力空間は SHA-256 の 32 byte = 256 bit、表記長が短い割に衝突は事実上ない
//   - GENESIS（チェーン先頭）のみ生 string "GENESIS" を保持（区別子として可読性優先）
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

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
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
    /// Postgres 等の永続バックエンド由来のエラー（PostgresAuditStore で発生）。
    Backend(String),
    /// チェーン整合性違反（verify_chain でのみ使用）。
    Integrity(String),
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreError::Serialize(s) => write!(f, "serialize: {}", s),
            StoreError::LockPoisoned => write!(f, "lock poisoned"),
            StoreError::Backend(s) => write!(f, "backend: {}", s),
            StoreError::Integrity(s) => write!(f, "integrity: {}", s),
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
pub(crate) fn canonical_bytes(input: &AppendInput) -> Result<Vec<u8>, StoreError> {
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

/// hash_chain は prev_id + canonical_bytes から SHA-256 を計算し base64（URL-safe / padding なし）で返す。
/// FR-T1-AUDIT-001 要件「ハッシュは SHA-256, base64 エンコード」を満たす。
fn hash_chain(prev_id: &str, canon: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(prev_id.as_bytes());
    h.update(canon);
    URL_SAFE_NO_PAD.encode(h.finalize())
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


// 単体テストは src/store_tests.rs に分離（src/CLAUDE.md「1 ファイル 500 行以内」遵守）。
// `#[path]` で sibling として明示し、Rust 既定の sub-module 規約（store/store_tests.rs）を回避する。
#[cfg(test)]
#[path = "store_tests.rs"]
mod store_tests;
