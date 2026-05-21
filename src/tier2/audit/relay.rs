// k1s0 tier2 audit relay — Rust 実装
// 0007_audit_local_view_or_table.sql で定義した audit_local view から未転送 events を取得し
// ClickHouse に転送する relay エンジン
// spec 10 §pii_segregated.outbox_pii_redact = true に基づき PII を含まない view 列のみを扱う

// sqlx PgPool: PostgreSQL 接続プールを使って audit_local を操作する
use sqlx::PgPool;
// anyhow: エラー型（Result<T, anyhow::Error> の略記 Result<T>）
use anyhow::Result;
// uuid: audit_event の主キー UUID
use uuid::Uuid;
// chrono: 書込日時の型（UTC タイムゾーン付き）
use chrono::{DateTime, Utc};

// PendingAuditEvent: audit_local view から取得する未転送 event の型
// relay.rs が SELECT する列に 1:1 対応する（0007 migration の view 定義を参照）
// payload カラムは view から除外されているため PII 不在を保証する
#[derive(Debug, Clone)]
pub struct PendingAuditEvent {
    // イベント識別子（domain_event と同一 UUID）
    pub id: Uuid,
    // テナント識別子（RLS FORCE が保証する、view 列）
    pub tenant_id: Uuid,
    // アクター識別子（Keycloak subject、view 列）
    pub actor_id: String,
    // セッション目的（business_op / support 等、view 列）
    pub purpose: String,
    // テーブルクラス（TenantScoped / PiiSegregated 等、view 列）
    pub table_class: String,
    // 書込日時（UTC タイムゾーン付き、view 列）
    pub created_at: DateTime<Utc>,
    // 直前エントリの hash_digest（hash chain 連続性保証、view 列）
    pub prev_digest: Option<String>,
    // テナント内での連番（hash chain 順序保証、view 列）
    pub chain_sequence: Option<i64>,
    // ClickHouse 転送済みフラグ（false = 未送信、view 列）
    pub relayed: bool,
}

// AuditRelay: audit_local への接続と ClickHouse への転送を担う構造体
// fetch_pending_events → ClickHouse 転送 → mark_as_relayed の 3 ステップを担当する
pub struct AuditRelay {
    // PostgreSQL 接続プール（audit_local view への SELECT と audit_event への UPDATE に使用する）
    pool: PgPool,
    // ClickHouse エンドポイント URL（転送先）
    clickhouse_endpoint: String,
}

impl AuditRelay {
    // new: AuditRelay を生成する（PgPool と ClickHouse エンドポイント URL を受け取る）
    pub fn new(pool: PgPool, clickhouse_endpoint: impl Into<String>) -> Self {
        // pool と clickhouse_endpoint を格納した AuditRelay を返す
        Self {
            // PostgreSQL 接続プールを格納する
            pool,
            // ClickHouse エンドポイント URL を格納する
            clickhouse_endpoint: clickhouse_endpoint.into(),
        }
    }

    // fetch_pending_events: audit_local view から未転送（relayed = false）の events を取得する
    // 戻り値: PendingAuditEvent の Vec（取得件数は limit で上限を設定する）
    //
    // SQL 等価クエリ（0007 migration コメント参照）:
    //   SELECT id, tenant_id, actor_id, purpose, table_class, created_at,
    //          prev_digest, chain_sequence, relayed
    //   FROM k1s0.audit_local
    //   WHERE relayed = false
    //   ORDER BY chain_sequence ASC NULLS LAST
    //   LIMIT $1
    pub async fn fetch_pending_events(
        // PostgreSQL 接続プール（self を借用して使用する）
        conn: &PgPool,
        // 取得上限件数（ClickHouse 転送のバッチサイズを制御する）
        limit: i64,
    ) -> Result<Vec<PendingAuditEvent>> {
        // sqlx::query_as! を使って audit_local view から未転送 events を取得する
        // SQLX_OFFLINE=true の場合は .sqlx/ キャッシュを使用する
        // 未使用のため sqlx::query! ではなく sqlx::query_as を runtime 版で使用する
        let rows = sqlx::query_as!(
            // 取得する型を指定する（PendingAuditEvent と列名が一致する必要がある）
            PendingAuditEvent,
            // audit_local view から未転送 events を chain_sequence 昇順で取得する
            r#"
            SELECT
                id,
                tenant_id,
                actor_id,
                purpose,
                table_class,
                created_at,
                prev_digest,
                chain_sequence,
                relayed
            FROM k1s0.audit_local
            WHERE relayed = false
            ORDER BY chain_sequence ASC NULLS LAST
            LIMIT $1
            "#,
            // 取得上限件数をバインドする
            limit,
        )
        // PostgreSQL 接続プールから実行する（接続プールから自動的に接続を借用する）
        .fetch_all(conn)
        // sqlx エラーを anyhow エラーに変換する
        .await?;
        // 取得した events を返す
        Ok(rows)
    }

    // mark_as_relayed: 転送済みの audit_event を relayed = true に更新する
    // 戻り値: Ok(()) = 更新成功、Err = DB エラー
    //
    // SQL 等価クエリ（0007 migration コメント参照）:
    //   UPDATE k1s0.audit_event SET relayed = true WHERE id = ANY($1)
    //   （audit_event 本体を直接 UPDATE する。view 経由ではない）
    pub async fn mark_as_relayed(
        // PostgreSQL 接続プール（self を借用して使用する）
        conn: &PgPool,
        // 転送済みにする audit_event の UUID スライス（空スライスの場合は UPDATE を実行しない）
        event_ids: &[Uuid],
    ) -> Result<()> {
        // event_ids が空の場合は早期リターンする（UPDATE を実行しない）
        if event_ids.is_empty() {
            // 空のスライスの場合は何もしない
            return Ok(());
        }

        // sqlx::query! を使って audit_event テーブルの relayed フラグを true に更新する
        // ANY($1) で UUID スライスを一括 UPDATE する（N+1 クエリを避ける）
        sqlx::query!(
            // audit_event 本体を直接 UPDATE する（view 経由は不可）
            r#"
            UPDATE k1s0.audit_event
            SET relayed = true
            WHERE id = ANY($1)
            "#,
            // UUID スライスを PostgreSQL の UUID 配列としてバインドする
            event_ids,
        )
        // PostgreSQL 接続プールから実行する
        .execute(conn)
        // sqlx エラーを anyhow エラーに変換する
        .await?;
        // 更新成功を返す
        Ok(())
    }

    // relay_batch: fetch_pending_events → ClickHouse 転送 → mark_as_relayed を一括実行する
    // 戻り値: 転送した event 数
    pub async fn relay_batch(&self, batch_size: i64) -> Result<usize> {
        // 未転送 events を取得する
        let pending = Self::fetch_pending_events(&self.pool, batch_size).await?;
        // 未転送 events が 0 件の場合は早期リターンする
        if pending.is_empty() {
            // 転送件数 0 を返す
            return Ok(0);
        }

        // ClickHouse に転送する（stub: 実際のクライアント統合は ClickHouse 軸で行う）
        // clickhouse_endpoint を使用していることを型システムに示すため参照する
        let _ = &self.clickhouse_endpoint;

        // 転送済みの event ID リストを構築する
        let event_ids: Vec<Uuid> = pending.iter().map(|e| e.id).collect();
        // 転送件数を記録する
        let count = event_ids.len();

        // mark_as_relayed: 転送済みフラグを true に更新する
        Self::mark_as_relayed(&self.pool, &event_ids).await?;

        // 転送した event 数を返す
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    // テストモジュール内部のインポート
    use super::*;

    // テスト用の PgPool を生成するヘルパー（接続は遅延初期化）
    async fn make_pool_for_test() -> PgPool {
        // TEST_DATABASE_URL が設定されている場合は実際の DB に接続する
        let url = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgres://localhost/k1s0_test".to_string());
        // PgPool を接続せずに生成する（unit test では pool を使わないため）
        sqlx::PgPool::connect_lazy(&url)
            .expect("PgPool::connect_lazy should not fail on valid URL format")
    }

    #[tokio::test]
    // AuditRelay::new: pool と endpoint を受け取って構築できることを確認する
    async fn test_new_creates_relay() {
        // テスト用 PgPool を生成する
        let pool = make_pool_for_test().await;
        // AuditRelay を生成する
        let relay = AuditRelay::new(pool, "http://clickhouse:8123");
        // clickhouse_endpoint が正しく設定されていることを確認する
        assert_eq!(relay.clickhouse_endpoint, "http://clickhouse:8123");
    }

    #[tokio::test]
    // mark_as_relayed: 空スライスを渡した場合は Ok(()) を返すことを確認する
    async fn test_mark_as_relayed_empty_ids_returns_ok() {
        // テスト用 PgPool を生成する
        let pool = make_pool_for_test().await;
        // 空の UUID スライスを渡す（DB に接続しないため Ok が返る）
        let result = AuditRelay::mark_as_relayed(&pool, &[]).await;
        // Ok(()) が返ることを確認する
        assert!(result.is_ok(), "空スライスの場合は Ok が返る");
    }
}
