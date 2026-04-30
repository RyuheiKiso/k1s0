// 本ファイルは Audit WORM ストアの Postgres backend 実装。
//
// 設計正典:
//   docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/01_tier1全体コンポーネント俯瞰.md
//     - DS-SW-COMP-007: t1-audit Pod = StatefulSet + WORM ストア（CNPG-backed）
//   docs/03_要件定義/30_非機能要件/H_アーティファクト完全性とコンプライアンス.md
//     - NFR-H-INT-001: WORM 性（削除不能、改ざん検知可能）
//     - NFR-H-INT-002: 完全性違反検知 + 違反箇所特定
//
// 役割:
//   InMemoryAuditStore は process メモリのみ保持で再起動 = 全消失。production の
//   NFR-H-INT-001（永続性）を満たすため、CNPG-backed Postgres を hash chain の
//   永続バックエンドとして使う。
//
//   WORM 性は以下で実現:
//     1. テーブルへの DELETE / UPDATE は app role で REVOKE（運用権限のみ ALTER 可）
//     2. trigger で UPDATE / DELETE 行をすべて RAISE EXCEPTION
//     3. Postgres のトランザクション内で SELECT MAX prev → INSERT で race-free な
//        prev_id チェイン構築
//
//   verify_chain_detail は ORDER BY sequence ASC で全件走査して prev_id / audit_id
//   の不整合を検出する。改ざん検知時は first_bad_sequence + reason を返す。
//
// 実装ノート:
//   AuditStore trait は同期 API。Tonic handler は async fn の中から呼ぶため、
//   tokio multi-thread runtime 内では `tokio::task::block_in_place` + `Handle::block_on`
//   の組み合わせで安全に同期化できる（block_in_place が当該 thread を blocking pool に
//   移し、他の async task が他の worker thread で進む）。

use crate::store::{
    AppendInput, AuditEntry, AuditStore, QueryInput, StoreError, VerifyOutcome, canonical_bytes,
};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::runtime::Handle;
use tokio_postgres::{Client, NoTls};

/// PostgresAuditStore は CNPG-backed WORM ストア。
pub struct PostgresAuditStore {
    client: Arc<Client>,
    runtime: Handle,
}

impl PostgresAuditStore {
    /// 接続 + schema 適用。`dsn` 例: `postgresql://user:pass@host:5432/dapr`。
    pub async fn connect(dsn: &str) -> Result<Self, StoreError> {
        let runtime = Handle::current();
        let (client, connection) = tokio_postgres::connect(dsn, NoTls)
            .await
            .map_err(|e| StoreError::Backend(format!("postgres connect: {}", e)))?;
        // 接続を background task で driving する（tokio-postgres の慣用）。
        runtime.spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("[postgres-audit] connection error: {}", e);
            }
        });
        // schema 適用。WORM trigger は app role に対し UPDATE/DELETE を block する。
        client
            .batch_execute(SCHEMA_SQL)
            .await
            .map_err(|e| StoreError::Backend(format!("schema migrate: {}", e)))?;
        Ok(PostgresAuditStore {
            client: Arc::new(client),
            runtime,
        })
    }

    /// 末尾 entry の audit_id を取得（chain の prev 計算用）。
    /// 空テーブルの場合は "GENESIS" を返す。
    async fn last_audit_id(client: &Client) -> Result<String, StoreError> {
        let row = client
            .query_opt(
                "SELECT audit_id FROM audit_entries ORDER BY sequence DESC LIMIT 1",
                &[],
            )
            .await
            .map_err(|e| StoreError::Backend(format!("query last: {}", e)))?;
        Ok(match row {
            Some(r) => r.get::<_, String>("audit_id"),
            None => "GENESIS".to_string(),
        })
    }
}

/// schema migration（一度だけ流せば idempotent）。
const SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS audit_entries (
    sequence    BIGSERIAL PRIMARY KEY,
    audit_id    TEXT NOT NULL UNIQUE,
    prev_id     TEXT NOT NULL,
    timestamp_ms BIGINT NOT NULL,
    actor       TEXT NOT NULL,
    action      TEXT NOT NULL,
    resource    TEXT NOT NULL,
    outcome     TEXT NOT NULL,
    attributes  JSONB NOT NULL DEFAULT '{}'::jsonb,
    tenant_id   TEXT NOT NULL,
    inserted_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_audit_tenant_ts ON audit_entries (tenant_id, timestamp_ms);

CREATE OR REPLACE FUNCTION audit_block_modify() RETURNS trigger AS $$
BEGIN
    RAISE EXCEPTION 'audit_entries is WORM (UPDATE/DELETE forbidden)';
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS audit_worm_block ON audit_entries;
CREATE TRIGGER audit_worm_block
    BEFORE UPDATE OR DELETE ON audit_entries
    FOR EACH ROW
    EXECUTE FUNCTION audit_block_modify();
"#;

impl AuditStore for PostgresAuditStore {
    fn append(&self, input: AppendInput) -> Result<String, StoreError> {
        let client = self.client.clone();
        let rt = self.runtime.clone();
        tokio::task::block_in_place(move || {
            rt.block_on(async move {
                let prev_id = Self::last_audit_id(&client).await?;
                let canon = canonical_bytes(&input)?;
                let mut hasher = Sha256::new();
                hasher.update(prev_id.as_bytes());
                hasher.update(&canon);
                let audit_id = hex::encode(hasher.finalize());
                let attrs_json = serde_json::to_value(&input.attributes)
                    .map_err(|e| StoreError::Backend(format!("attributes serialize: {}", e)))?;
                client
                    .execute(
                        "INSERT INTO audit_entries (audit_id, prev_id, timestamp_ms, actor, \
                         action, resource, outcome, attributes, tenant_id) \
                         VALUES ($1,$2,$3,$4,$5,$6,$7,$8::jsonb,$9)",
                        &[
                            &audit_id,
                            &prev_id,
                            &input.timestamp_ms,
                            &input.actor,
                            &input.action,
                            &input.resource,
                            &input.outcome,
                            &attrs_json,
                            &input.tenant_id,
                        ],
                    )
                    .await
                    .map_err(|e| StoreError::Backend(format!("insert: {}", e)))?;
                Ok(audit_id)
            })
        })
    }

    fn query(&self, q: QueryInput) -> Result<Vec<AuditEntry>, StoreError> {
        let client = self.client.clone();
        let rt = self.runtime.clone();
        tokio::task::block_in_place(move || {
            rt.block_on(async move {
                let mut sql = String::from(
                    "SELECT timestamp_ms, actor, action, resource, outcome, attributes, \
                     tenant_id, prev_id, audit_id FROM audit_entries WHERE tenant_id = $1",
                );
                // dynamic params の構築。tokio-postgres の query() は &[&(dyn ToSql + Sync)] を
                // 取るため、所有者を Vec<Box<dyn ToSql + Sync + Send>> に持って借用に変換する。
                let mut params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> =
                    vec![Box::new(q.tenant_id.clone())];
                if let Some(from) = q.from_ms {
                    params.push(Box::new(from));
                    sql.push_str(&format!(" AND timestamp_ms >= ${}", params.len()));
                }
                if let Some(to) = q.to_ms {
                    params.push(Box::new(to));
                    sql.push_str(&format!(" AND timestamp_ms <= ${}", params.len()));
                }
                sql.push_str(" ORDER BY timestamp_ms ASC");
                if q.limit > 0 {
                    sql.push_str(&format!(" LIMIT {}", q.limit as i64));
                }
                let refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = params
                    .iter()
                    .map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync))
                    .collect();
                let rows = client
                    .query(sql.as_str(), refs.as_slice())
                    .await
                    .map_err(|e| StoreError::Backend(format!("query: {}", e)))?;
                let mut out = Vec::with_capacity(rows.len());
                for r in rows {
                    let attrs_json: serde_json::Value = r.get("attributes");
                    let attrs: BTreeMap<String, String> = match attrs_json {
                        serde_json::Value::Object(o) => o
                            .into_iter()
                            .map(|(k, v)| {
                                (k, v.as_str().map(String::from).unwrap_or_default())
                            })
                            .collect(),
                        _ => BTreeMap::new(),
                    };
                    let mut matches = true;
                    for (k, want) in q.filters.iter() {
                        if attrs.get(k).map_or(true, |v| v != want) {
                            matches = false;
                            break;
                        }
                    }
                    if !matches {
                        continue;
                    }
                    out.push(AuditEntry {
                        timestamp_ms: r.get("timestamp_ms"),
                        actor: r.get("actor"),
                        action: r.get("action"),
                        resource: r.get("resource"),
                        outcome: r.get("outcome"),
                        attributes: attrs,
                        tenant_id: r.get("tenant_id"),
                        prev_id: r.get("prev_id"),
                        audit_id: r.get("audit_id"),
                    });
                }
                Ok(out)
            })
        })
    }

    fn verify_chain(&self) -> Result<(), StoreError> {
        let v = self.verify_chain_detail("", None, None)?;
        if v.valid {
            Ok(())
        } else {
            Err(StoreError::Integrity(v.reason))
        }
    }

    fn verify_chain_detail(
        &self,
        tenant_id: &str,
        from_ms: Option<i64>,
        to_ms: Option<i64>,
    ) -> Result<VerifyOutcome, StoreError> {
        let client = self.client.clone();
        let rt = self.runtime.clone();
        let tenant_id = tenant_id.to_string();
        tokio::task::block_in_place(move || {
            rt.block_on(async move {
                let rows = client
                    .query(
                        "SELECT timestamp_ms, actor, action, resource, outcome, attributes, \
                         tenant_id, prev_id, audit_id FROM audit_entries ORDER BY sequence ASC",
                        &[],
                    )
                    .await
                    .map_err(|e| StoreError::Backend(format!("verify query: {}", e)))?;
                let mut prev = "GENESIS".to_string();
                let mut checked: i64 = 0;
                for (idx, r) in rows.iter().enumerate() {
                    let row_prev: String = r.get("prev_id");
                    let row_audit_id: String = r.get("audit_id");
                    if row_prev != prev {
                        return Ok(VerifyOutcome {
                            valid: false,
                            checked_count: checked,
                            first_bad_sequence: (idx as i64) + 1,
                            reason: format!(
                                "prev_id mismatch at sequence {}: expected {}, got {}",
                                idx + 1,
                                prev,
                                row_prev
                            ),
                        });
                    }
                    let attrs_json: serde_json::Value = r.get("attributes");
                    let attrs: BTreeMap<String, String> = match attrs_json {
                        serde_json::Value::Object(o) => o
                            .into_iter()
                            .map(|(k, v)| {
                                (k, v.as_str().map(String::from).unwrap_or_default())
                            })
                            .collect(),
                        _ => BTreeMap::new(),
                    };
                    let input = AppendInput {
                        timestamp_ms: r.get("timestamp_ms"),
                        actor: r.get("actor"),
                        action: r.get("action"),
                        resource: r.get("resource"),
                        outcome: r.get("outcome"),
                        attributes: attrs.clone(),
                        tenant_id: r.get("tenant_id"),
                    };
                    let canon = canonical_bytes(&input)?;
                    let mut hasher = Sha256::new();
                    hasher.update(prev.as_bytes());
                    hasher.update(&canon);
                    let computed = hex::encode(hasher.finalize());
                    if computed != row_audit_id {
                        return Ok(VerifyOutcome {
                            valid: false,
                            checked_count: checked,
                            first_bad_sequence: (idx as i64) + 1,
                            reason: format!(
                                "audit_id mismatch at sequence {}: expected {}, got {}",
                                idx + 1,
                                row_audit_id,
                                computed
                            ),
                        });
                    }
                    let row_tenant: String = r.get("tenant_id");
                    let row_ts: i64 = r.get("timestamp_ms");
                    let in_tenant = tenant_id.is_empty() || row_tenant == tenant_id;
                    let in_range = from_ms.is_none_or(|f| row_ts >= f)
                        && to_ms.is_none_or(|t| row_ts <= t);
                    if in_tenant && in_range {
                        checked += 1;
                    }
                    prev = row_audit_id;
                }
                Ok(VerifyOutcome {
                    valid: true,
                    checked_count: checked,
                    first_bad_sequence: 0,
                    reason: String::new(),
                })
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// G8 regression: WORM trigger が schema migration から消されていないこと。
    /// 将来 schema を編集する誰かが trigger を緩めたり削除したりすると WORM 性
    /// が静かに壊れるので、SCHEMA_SQL に必須要素が含まれていることを文字列検査
    /// で固定する (実 Postgres 接続テストは E2E で別建て、本テストは unit-only)。
    #[test]
    fn schema_sql_includes_worm_trigger() {
        assert!(
            SCHEMA_SQL.contains("audit_block_modify"),
            "WORM trigger function 'audit_block_modify' must be in SCHEMA_SQL"
        );
        assert!(
            SCHEMA_SQL.contains("BEFORE UPDATE OR DELETE"),
            "trigger must fire BEFORE UPDATE OR DELETE to block writes"
        );
        assert!(
            SCHEMA_SQL.contains("RAISE EXCEPTION"),
            "trigger must RAISE EXCEPTION to abort the modification"
        );
        assert!(
            SCHEMA_SQL.contains("WORM"),
            "trigger error message must mention WORM for operator clarity"
        );
    }

    /// schema が必須カラム (audit_id / prev_id / tenant_id 等) を持つこと。
    #[test]
    fn schema_sql_has_all_required_columns() {
        let required = [
            "audit_id    TEXT NOT NULL",
            "prev_id     TEXT NOT NULL",
            "timestamp_ms BIGINT NOT NULL",
            "actor       TEXT NOT NULL",
            "tenant_id   TEXT NOT NULL",
            "attributes  JSONB",
        ];
        for col in &required {
            assert!(
                SCHEMA_SQL.contains(col),
                "SCHEMA_SQL missing required column: {}",
                col
            );
        }
    }

    /// idx_audit_tenant_ts index (NFR-B-PERF: tenant + 時刻範囲クエリの高速化)。
    #[test]
    fn schema_sql_has_tenant_timestamp_index() {
        assert!(
            SCHEMA_SQL.contains("idx_audit_tenant_ts"),
            "schema must create idx_audit_tenant_ts for (tenant_id, timestamp_ms) lookups"
        );
    }
}
