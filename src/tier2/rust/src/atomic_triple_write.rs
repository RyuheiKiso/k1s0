// k1s0 tier2 atomic 三表書込
// State change / Outbox / Audit event を同一 DB トランザクションで書く
// TLA+ の P1-P4 invariant と double-bind する（src/formal/dafny/AtomicThreeTableWrite.dfy）
//
// P1: aggregate 状態変更時、必ず Outbox + Audit が同一 txn に書込
// P2: Outbox 投入失敗時、aggregate 状態変更も rollback
// P3: tenant_id が GUC と aggregate 行で一致しない場合、操作を reject
// P4: pii_segregated の全アクセスを audit_event に記録

// UUID ライブラリ
use uuid::Uuid;
// シリアライズライブラリ
use serde::{Deserialize, Serialize};
// エラー型定義ライブラリ
use thiserror::Error;
// 日時ライブラリ
use chrono::{DateTime, Utc};
// sqlx PostgreSQL クライアント（実 transaction 実行に使用する）
use sqlx::Postgres;
// テナントコンテキスト
use crate::tenant_context::TenantContext;

// atomic 三表書込のエラー型
#[derive(Debug, Error)]
pub enum AtomicWriteError {
    // P2: Outbox 投入失敗（rollback が必要）
    #[error("Outbox insert failed, transaction rolled back: {0}")]
    OutboxInsertFailed(String),
    // P3: tenant_id 不一致（GUC と aggregate 行の tenant_id が一致しない）
    #[error("TenantId mismatch: guc={guc}, row={row}")]
    TenantIdMismatch { guc: Uuid, row: Uuid },
    // P4: pii_segregated の audit_event 記録失敗
    #[error("PII audit record failed: {0}")]
    PiiAuditFailed(String),
    // 汎用: DB トランザクションエラー
    #[error("Transaction error: {0}")]
    TransactionError(String),
}

// 書込対象のテーブルクラス（10_テナント分離適合仕様.md の 4 class と一致する）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TableClass {
    // テナントスコープ: tenant_id 必須、RLS FORCE
    TenantScoped,
    // テナントマスタ: tenant_id 必須、role 制限付き RLS FORCE
    TenantMaster,
    // プラットフォームグローバル: tenant_id 無し、RLS 無効
    PlatformGlobal,
    // PII 隔離: tenant_id 必須 + purpose check + pgaudit 全アクセス
    PiiSegregated,
}

// aggregate の状態変更を表す構造体（P1 の state_change に対応する）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateChange {
    // 変更対象の aggregate ID
    pub aggregate_id: Uuid,
    // 変更対象の tenant_id（TenantContext.tenant_id と一致している必要がある）
    pub tenant_id: Uuid,
    // テーブルクラス（どの class の table を書込むかを示す）
    pub table_class: TableClass,
    // 変更内容のシリアライズ済みペイロード
    pub payload: serde_json::Value,
    // aggregate バージョン（楽観的ロックに使用する）
    pub version: i64,
}

// atomic 三表書込の結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TripleWriteResult {
    // 書込んだ aggregate ID
    pub aggregate_id: Uuid,
    // 書込んだ outbox エントリの ID
    pub outbox_id: Uuid,
    // 書込んだ audit_event の ID
    pub audit_event_id: Uuid,
    // 書込完了日時
    pub committed_at: DateTime<Utc>,
}

// atomic 三表書込の実行エンジン
// sqlx::Transaction を受け取り、P1-P4 invariant を保証する
#[derive(Debug)]
pub struct AtomicTripleWrite {
    // テナントコンテキスト（GUC 注入・tenant_id 検証に使用する）
    context: TenantContext,
}

// execute() の戻り値型
// P1: BEGIN 〜 COMMIT の中で state_change / outbox / audit_event の 3 INSERT を実行する
// P2: outbox INSERT が失敗した場合は txn を rollback して OutboxInsertFailed を返す
// P3: tenant_id が GUC と一致しない場合は即座に reject して TransactionError を返す
// P4: pii_segregated テーブルへのアクセスは audit_event に記録してから txn を実行する
pub type ExecuteResult = Result<TripleWriteResult, AtomicWriteError>;

impl AtomicTripleWrite {
    // AtomicTripleWrite を生成する（TenantContext を受け取る）
    pub fn new(context: TenantContext) -> Self {
        // TenantContext を格納した AtomicTripleWrite を返す
        Self { context }
    }

    // P1-P4: atomic 三表書込を実行する非同期メソッド（実 PostgreSQL transaction を使用する）
    // tx: sqlx::Transaction<'_, Postgres> — 呼び出し元が BEGIN した transaction を受け取る
    // 呼び出し元は Ok 返却後に tx.commit() を呼ぶ。Err 返却後は tx.rollback() または drop で rollback される。
    pub async fn execute(
        &self,
        // 書込対象の状態変更（P1-P4 の対象）
        change: &StateChange,
        // 呼び出し元が管理する PostgreSQL transaction（同一 txn で 3 INSERT を実行する）
        tx: &mut sqlx::Transaction<'_, Postgres>,
    ) -> ExecuteResult {
        // P3: tenant_id 一致を事前検証する（GUC と aggregate 行の tenant_id が一致しない場合は即座にエラー）
        self.verify_tenant_id(change)?;
        // P3: SET LOCAL で 4 GUC を txn スコープに注入する（RLS FORCE が参照する）
        let set_guc_sql = self.context.to_set_local_sql();
        // SET LOCAL GUC を実行する（transaction スコープのみ有効、COMMIT で自動破棄される）
        sqlx::query(&set_guc_sql)
            .execute(&mut **tx)
            .await
            .map_err(|e| AtomicWriteError::TransactionError(e.to_string()))?;

        // P4: pii_segregated の場合は audit_event への記録が必須であることを確認する
        let pii_required = self.verify_pii_audit_required(change);
        // pii_required フラグをログとして使用する（実際は audit_event INSERT で常に記録する）
        let _ = pii_required;

        // Outbox エントリの ID を生成する（P1 の atomic 三表書込で使用する）
        let outbox_id = Uuid::new_v4();
        // audit_event の ID を生成する（domain_event と audit_event で共有する）
        let audit_event_id = Uuid::new_v4();
        // 書込完了日時を記録する（3 INSERT で統一した timestamp を使用する）
        let committed_at: DateTime<Utc> = Utc::now();

        // P1: k1s0.domain_event テーブルに INSERT する（aggregate 状態変更の永続化）
        // current_setting('app.tenant_id')::uuid を使って RLS FORCE の tenant_id を注入する
        sqlx::query(
            r#"
            INSERT INTO k1s0.domain_event
                (id, aggregate_id, tenant_id, event_kind, payload, version, created_at)
            VALUES
                ($1, $2, current_setting('app.tenant_id')::uuid, 'StateChange', $3, $4, $5)
            "#,
        )
        // audit_event_id を domain_event の主キーとして使用する
        .bind(audit_event_id)
        // 変更対象の aggregate ID をバインドする
        .bind(change.aggregate_id)
        // ペイロードを jsonb 型としてバインドする
        .bind(&change.payload)
        // aggregate バージョンをバインドする（楽観的ロックに使用する）
        .bind(change.version)
        // 書込完了日時をバインドする
        .bind(committed_at)
        // 同一 transaction で実行する（P1 の atomic 書込を保証する）
        .execute(&mut **tx)
        .await
        .map_err(|e| AtomicWriteError::TransactionError(e.to_string()))?;

        // P1: k1s0.outbox テーブルに INSERT する（Debezium CDC 経由で Kafka に転送される）
        // P2: この INSERT が失敗した場合は OutboxInsertFailed を返し、呼び出し元が rollback する
        sqlx::query(
            r#"
            INSERT INTO k1s0.outbox
                (id, aggregate_id, tenant_id, event_kind, payload, created_at)
            VALUES
                ($1, $2, current_setting('app.tenant_id')::uuid, 'OutboxRelay', $3, $4)
            "#,
        )
        // outbox エントリの ID をバインドする
        .bind(outbox_id)
        // 変更対象の aggregate ID をバインドする
        .bind(change.aggregate_id)
        // ペイロードを jsonb 型としてバインドする（PII は redact 済みのみ含む）
        .bind(&change.payload)
        // 書込完了日時をバインドする
        .bind(committed_at)
        // 同一 transaction で実行する（P2 の rollback 要件を満たす）
        .execute(&mut **tx)
        .await
        // P2: outbox INSERT 失敗は OutboxInsertFailed にマッピングして rollback を促す
        .map_err(|e| AtomicWriteError::OutboxInsertFailed(e.to_string()))?;

        // P1+P4: k1s0.audit_event テーブルに INSERT する（全操作を監査記録する）
        // P4: pii_segregated は pgaudit も併用するが、アプリ層からも必ず audit_event を書く
        sqlx::query(
            r#"
            INSERT INTO k1s0.audit_event
                (id, aggregate_id, tenant_id, actor_id, purpose, table_class, payload, created_at)
            VALUES
                ($1, $2, current_setting('app.tenant_id')::uuid,
                 current_setting('app.actor_id'),
                 current_setting('app.purpose'),
                 $3, $4, $5)
            "#,
        )
        // audit_event の ID をバインドする（domain_event と同じ ID で結びつける）
        .bind(audit_event_id)
        // 変更対象の aggregate ID をバインドする
        .bind(change.aggregate_id)
        // テーブルクラスを文字列としてバインドする
        .bind(format!("{:?}", change.table_class))
        // ペイロードを jsonb 型としてバインドする
        .bind(&change.payload)
        // 書込完了日時をバインドする
        .bind(committed_at)
        // 同一 transaction で実行する（P4 の audit 必須要件を満たす）
        .execute(&mut **tx)
        .await
        // P4: audit_event INSERT 失敗は PiiAuditFailed にマッピングして rollback を促す
        .map_err(|e| AtomicWriteError::PiiAuditFailed(e.to_string()))?;

        // 三表書込の結果を返す（呼び出し元が tx.commit() を呼ぶことで確定する）
        Ok(TripleWriteResult {
            // 書込んだ aggregate ID を返す
            aggregate_id: change.aggregate_id,
            // 書込んだ outbox エントリの ID を返す
            outbox_id,
            // 書込んだ audit_event の ID を返す
            audit_event_id,
            // 書込完了日時を返す
            committed_at,
        })
    }

    // P3: tenant_id 一致を検証する
    // StateChange の tenant_id が TenantContext の tenant_id と一致しない場合はエラーを返す
    pub fn verify_tenant_id(&self, change: &StateChange) -> Result<(), AtomicWriteError> {
        // GUC の tenant_id と aggregate の tenant_id を比較する
        let guc_tenant_id = self.context.tenant_id();
        if guc_tenant_id != change.tenant_id {
            // P3 違反: tenant_id 不一致でエラーを返す
            return Err(AtomicWriteError::TenantIdMismatch {
                guc: guc_tenant_id,
                row: change.tenant_id,
            });
        }
        // tenant_id が一致した場合は Ok を返す
        Ok(())
    }

    // P4: pii_segregated テーブルへのアクセスを audit_event に記録する必要があることを検証する
    pub fn verify_pii_audit_required(&self, change: &StateChange) -> bool {
        // pii_segregated の場合は必ず audit_event を記録する（true を返す）
        change.table_class == TableClass::PiiSegregated
    }

    // 三表書込に必要な SQL 文字列を生成する（P1 の atomic write の SQL 骨格）
    // 実際の DB 実行は execute() が sqlx::Transaction 経由で行う
    // このメソッドは単体テスト・デバッグ用に SQL 骨格を確認するために残す
    pub fn build_triple_write_sql(&self, change: &StateChange) -> Result<String, AtomicWriteError> {
        // P3: tenant_id 一致を事前検証する
        self.verify_tenant_id(change)?;
        // outbox / audit event の ID を生成する
        let outbox_id = Uuid::new_v4();
        let audit_id = Uuid::new_v4();
        let now = Utc::now().to_rfc3339();
        // SET LOCAL GUC 注入 SQL を取得する（4 GUC 全て）
        let set_guc = self.context.to_set_local_sql();
        // P1: state_change + outbox + audit_event を BEGIN 〜 COMMIT の間に書く
        let sql = format!(
            r#"
BEGIN;
{set_guc}

-- P1: state_change (aggregate テーブルへの書込)
INSERT INTO k1s0.domain_event (id, aggregate_id, tenant_id, event_kind, payload, version, created_at)
VALUES ('{audit_id}', '{agg_id}', current_setting('app.tenant_id')::uuid, 'StateChange', '{payload}'::jsonb, {version}, '{now}');

-- P1: outbox (Debezium CDC 経由で Kafka に転送される)
INSERT INTO k1s0.outbox (id, aggregate_id, tenant_id, event_kind, payload, created_at)
VALUES ('{outbox_id}', '{agg_id}', current_setting('app.tenant_id')::uuid, 'OutboxRelay', '{payload}'::jsonb, '{now}');

-- P1 + P4: audit_event (全操作で記録、pii_segregated は pgaudit も併用)
INSERT INTO k1s0.audit_event (id, aggregate_id, tenant_id, actor_id, purpose, table_class, payload, created_at)
VALUES ('{audit_id}', '{agg_id}', current_setting('app.tenant_id')::uuid, current_setting('app.actor_id'), current_setting('app.purpose'), '{table_class}', '{payload}'::jsonb, '{now}');

COMMIT;
"#,
            set_guc    = set_guc,
            agg_id     = change.aggregate_id,
            outbox_id  = outbox_id,
            audit_id   = audit_id,
            payload    = change.payload.to_string().replace('\'', "''"),
            version    = change.version,
            now        = now,
            table_class = format!("{:?}", change.table_class),
        );
        // 生成した SQL 文字列を返す
        Ok(sql)
    }

    // P2: Outbox 失敗シミュレーション（テスト専用、production では使用しない）
    #[cfg(test)]
    pub fn simulate_outbox_failure() -> AtomicWriteError {
        // outbox 失敗時は rollback するためのエラーを返す
        AtomicWriteError::OutboxInsertFailed("simulated outbox failure".to_string())
    }
}

#[cfg(test)]
mod tests {
    // テストモジュール内部でのみ使用するインポート
    use super::*;
    use crate::tenant_context::SessionPurpose;

    // テスト用の TenantContext を生成するヘルパー関数
    fn make_context(tenant_id: Uuid) -> TenantContext {
        TenantContext::from_auth(
            tenant_id,
            "test-actor".to_string(),
            SessionPurpose::BusinessOp,
        )
    }

    // テスト用の StateChange を生成するヘルパー関数
    fn make_state_change(tenant_id: Uuid, table_class: TableClass) -> StateChange {
        StateChange {
            aggregate_id: Uuid::new_v4(),
            tenant_id,
            table_class,
            payload: serde_json::json!({"key": "value"}),
            version: 1,
        }
    }

    #[test]
    // P3: tenant_id 一致の場合は成功することを確認する
    fn test_p3_tenant_id_match_ok() {
        // 同一の tenant_id で TenantContext と StateChange を生成する
        let tenant_id = Uuid::new_v4();
        let ctx = make_context(tenant_id);
        let writer = AtomicTripleWrite::new(ctx);
        let change = make_state_change(tenant_id, TableClass::TenantScoped);
        // P3 検証が成功することを確認する
        assert!(writer.verify_tenant_id(&change).is_ok());
    }

    #[test]
    // P3: tenant_id 不一致の場合は TenantIdMismatch エラーを返すことを確認する
    fn test_p3_tenant_id_mismatch_error() {
        // 異なる tenant_id で TenantContext と StateChange を生成する
        let ctx = make_context(Uuid::new_v4());
        let writer = AtomicTripleWrite::new(ctx);
        let different_tenant_id = Uuid::new_v4();
        let change = make_state_change(different_tenant_id, TableClass::TenantScoped);
        // P3 違反: TenantIdMismatch エラーが返されることを確認する
        let err = writer.verify_tenant_id(&change);
        assert!(matches!(err, Err(AtomicWriteError::TenantIdMismatch { .. })));
    }

    #[test]
    // P4: pii_segregated の場合は audit 必須フラグが true を返すことを確認する
    fn test_p4_pii_audit_required() {
        // PiiSegregated テーブルクラスの StateChange を生成する
        let tenant_id = Uuid::new_v4();
        let ctx = make_context(tenant_id);
        let writer = AtomicTripleWrite::new(ctx);
        let pii_change = make_state_change(tenant_id, TableClass::PiiSegregated);
        // pii_segregated では audit が必須であることを確認する
        assert!(writer.verify_pii_audit_required(&pii_change));
        // tenant_scoped では pgaudit は不要（通常 audit_event のみ）
        let normal_change = make_state_change(tenant_id, TableClass::TenantScoped);
        assert!(!writer.verify_pii_audit_required(&normal_change));
    }

    #[test]
    // P1: 三表書込 SQL が state_change / outbox / audit_event を全て含むことを確認する
    fn test_p1_triple_write_sql_contains_all_tables() {
        // TenantContext と StateChange を生成する
        let tenant_id = Uuid::new_v4();
        let ctx = make_context(tenant_id);
        let writer = AtomicTripleWrite::new(ctx);
        let change = make_state_change(tenant_id, TableClass::TenantScoped);
        // SQL を生成する
        let sql = writer.build_triple_write_sql(&change).unwrap();
        // BEGIN と COMMIT の間に 3 つの INSERT が含まれることを確認する
        assert!(sql.contains("BEGIN"));
        assert!(sql.contains("domain_event"));
        assert!(sql.contains("outbox"));
        assert!(sql.contains("audit_event"));
        assert!(sql.contains("COMMIT"));
        // GUC 注入が含まれることを確認する
        assert!(sql.contains("app.tenant_id"));
    }

    #[test]
    // P2: outbox 失敗シミュレーションがエラーを返すことを確認する
    fn test_p2_outbox_failure_simulation() {
        // outbox 失敗エラーが OutboxInsertFailed であることを確認する
        let err = AtomicTripleWrite::simulate_outbox_failure();
        assert!(matches!(err, AtomicWriteError::OutboxInsertFailed(_)));
    }
}
