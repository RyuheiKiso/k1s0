use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entity::saga_state::{SagaState, SagaStatus};
use crate::domain::entity::saga_step_log::{SagaStepLog, StepAction, StepStatus};
use crate::domain::repository::saga_repository::{SagaListParams, SagaRepository};

/// SagaPostgresRepository はPostgreSQL実装のSagaリポジトリ。
pub struct SagaPostgresRepository {
    pool: PgPool,
}

impl SagaPostgresRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SagaRepository for SagaPostgresRepository {
    async fn create(&self, state: &SagaState) -> anyhow::Result<()> {
        // CRIT-005 対応: トランザクション内で tenant_id をセッション変数に設定してから INSERT する
        let mut tx = self.pool.begin().await?;

        // テナント分離: RLS のために現在のテナントIDをセッション変数に設定する
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(&state.tenant_id)
            .execute(&mut *tx)
            .await?;

        sqlx::query(
            r#"
            INSERT INTO saga.saga_states
                (id, workflow_name, current_step, status, payload, correlation_id, initiated_by, error_message, tenant_id, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
        )
        .bind(state.saga_id)
        .bind(&state.workflow_name)
        .bind(state.current_step)
        .bind(state.status.to_string())
        .bind(&state.payload)
        .bind(&state.correlation_id)
        .bind(&state.initiated_by)
        .bind(&state.error_message)
        .bind(&state.tenant_id)
        .bind(state.created_at)
        .bind(state.updated_at)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(())
    }

    async fn update_with_step_log(
        &self,
        state: &SagaState,
        log: &SagaStepLog,
    ) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;

        // テナント分離: RLS のために現在のテナントIDをセッション変数に設定する
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(&state.tenant_id)
            .execute(&mut *tx)
            .await?;

        sqlx::query(
            r#"
            UPDATE saga.saga_states
            SET current_step = $2, status = $3, payload = $4, error_message = $5, updated_at = NOW()
            WHERE id = $1 AND tenant_id = $6
            "#,
        )
        .bind(state.saga_id)
        .bind(state.current_step)
        .bind(state.status.to_string())
        .bind(&state.payload)
        .bind(&state.error_message)
        .bind(&state.tenant_id)
        .execute(&mut *tx)
        .await?;

        // saga_step_logs にも tenant_id を挿入する
        sqlx::query(
            r#"
            INSERT INTO saga.saga_step_logs
                (id, saga_id, step_index, step_name, action, status, request_payload, response_payload, error_message, tenant_id, started_at, completed_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
        )
        .bind(log.id)
        .bind(log.saga_id)
        .bind(log.step_index)
        .bind(&log.step_name)
        .bind(log.action.to_string())
        .bind(log.status.to_string())
        .bind(&log.request_payload)
        .bind(&log.response_payload)
        .bind(&log.error_message)
        .bind(&state.tenant_id)
        .bind(log.started_at)
        .bind(log.completed_at)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(())
    }

    async fn update_status(
        &self,
        saga_id: Uuid,
        status: &SagaStatus,
        error_message: Option<String>,
        tenant_id: &str,
    ) -> anyhow::Result<()> {
        // CRIT-005 対応: トランザクション内で tenant_id をセッション変数に設定してから UPDATE する
        let mut tx = self.pool.begin().await?;

        // テナント分離: RLS のために現在のテナントIDをセッション変数に設定する
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;

        sqlx::query(
            r#"
            UPDATE saga.saga_states
            SET status = $2, error_message = $3, updated_at = NOW()
            WHERE id = $1 AND tenant_id = $4
            "#,
        )
        .bind(saga_id)
        .bind(status.to_string())
        .bind(error_message)
        .bind(tenant_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(())
    }

    async fn find_by_id(&self, saga_id: Uuid, tenant_id: &str) -> anyhow::Result<Option<SagaState>> {
        // CRIT-005 対応: トランザクション内で tenant_id をセッション変数に設定してから SELECT する
        let mut tx = self.pool.begin().await?;

        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;

        let row = sqlx::query_as::<_, SagaStateRow>(
            r#"
            SELECT id, workflow_name, current_step, status, payload, correlation_id, initiated_by, error_message, tenant_id, created_at, updated_at
            FROM saga.saga_states
            WHERE id = $1
            "#,
        )
        .bind(saga_id)
        .fetch_optional(&mut *tx)
        .await?;

        tx.commit().await?;

        row.map(|r| r.try_into()).transpose()
    }

    async fn find_step_logs(&self, saga_id: Uuid, tenant_id: &str) -> anyhow::Result<Vec<SagaStepLog>> {
        // CRIT-005 対応: トランザクション内で tenant_id をセッション変数に設定してから SELECT する
        let mut tx = self.pool.begin().await?;

        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;

        let rows = sqlx::query_as::<_, StepLogRow>(
            r#"
            SELECT id, saga_id, step_index, step_name, action, status, request_payload, response_payload, error_message, started_at, completed_at
            FROM saga.saga_step_logs
            WHERE saga_id = $1
            ORDER BY step_index, started_at
            "#,
        )
        .bind(saga_id)
        .fetch_all(&mut *tx)
        .await?;

        tx.commit().await?;

        rows.into_iter().map(|r| r.try_into()).collect()
    }

    /// Saga 一覧を取得する。
    /// cursor が指定された場合は keyset ページネーションを使用し、
    /// 指定されない場合は後方互換のため OFFSET ページネーションを使用する。
    /// keyset ページネーションは大規模データセットで OFFSET より高パフォーマンスを発揮する。
    async fn list(&self, params: &SagaListParams) -> anyhow::Result<(Vec<SagaState>, i32)> {
        // CRIT-005 対応: トランザクション内で tenant_id をセッション変数に設定してから SELECT する
        let mut tx = self.pool.begin().await?;

        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(&params.tenant_id)
            .execute(&mut *tx)
            .await?;

        // CRIT-3 監査対応: sqlx::QueryBuilder を使用してタイプセーフな動的クエリを生成する。
        let page = params.page.max(1);
        let page_size = params.page_size.max(1);

        // カウントクエリを QueryBuilder で構築する（keyset/OFFSET 共通）
        let mut count_qb = sqlx::QueryBuilder::new("SELECT COUNT(*)::int4 FROM saga.saga_states WHERE tenant_id = ");
        count_qb.push_bind(&params.tenant_id);

        if let Some(ref wn) = params.workflow_name {
            count_qb.push(" AND workflow_name = ");
            count_qb.push_bind(wn);
        }
        if let Some(ref s) = params.status {
            count_qb.push(" AND status = ");
            count_qb.push_bind(s.to_string());
        }
        if let Some(ref ci) = params.correlation_id {
            count_qb.push(" AND correlation_id = ");
            count_qb.push_bind(ci);
        }

        let total: i32 = count_qb.build_query_scalar().fetch_one(&mut *tx).await?;

        // データクエリを QueryBuilder で構築する
        let mut data_qb = sqlx::QueryBuilder::new(
            "SELECT id, workflow_name, current_step, status, payload, correlation_id, initiated_by, error_message, tenant_id, created_at, updated_at FROM saga.saga_states WHERE tenant_id = ",
        );
        data_qb.push_bind(&params.tenant_id);

        // フィルタ条件を追加する（keyset/OFFSET 共通）
        if let Some(ref wn) = params.workflow_name {
            data_qb.push(" AND workflow_name = ");
            data_qb.push_bind(wn);
        }
        if let Some(ref s) = params.status {
            data_qb.push(" AND status = ");
            data_qb.push_bind(s.to_string());
        }
        if let Some(ref ci) = params.correlation_id {
            data_qb.push(" AND correlation_id = ");
            data_qb.push_bind(ci);
        }

        // cursor が指定された場合は keyset ページネーションを使用する。
        // cursor 形式: "{created_at_unix_ms}_{id}"
        // (created_at, id) の複合比較で一意性を保証し、重複・欠損のないページングを実現する。
        if let Some(ref cursor) = params.cursor {
            // カーソルを unix_ms とIDに分解する
            if let Some((ts_str, id_str)) = cursor.split_once('_') {
                if let (Ok(ts_ms), Ok(cursor_id)) =
                    (ts_str.parse::<i64>(), uuid::Uuid::parse_str(id_str))
                {
                    // unix_ms をマイクロ秒単位の PostgreSQL TIMESTAMPTZ に変換する
                    let cursor_ts = chrono::DateTime::<chrono::Utc>::from_timestamp_millis(ts_ms);
                    if let Some(cursor_ts) = cursor_ts {
                        data_qb.push(" AND (created_at, id) < (");
                        data_qb.push_bind(cursor_ts);
                        data_qb.push(", ");
                        data_qb.push_bind(cursor_id);
                        data_qb.push(")");
                    }
                }
            }
            // keyset ページネーション: ORDER BY created_at DESC, id DESC
            data_qb.push(" ORDER BY created_at DESC, id DESC LIMIT ");
            data_qb.push_bind(page_size as i64);
        } else {
            // OFFSET ページネーション（後方互換）
            let offset = ((page - 1) * page_size) as i64;
            data_qb.push(" ORDER BY created_at DESC LIMIT ");
            data_qb.push_bind(page_size as i64);
            data_qb.push(" OFFSET ");
            data_qb.push_bind(offset);
        }

        let rows = data_qb
            .build_query_as::<SagaStateRow>()
            .fetch_all(&mut *tx)
            .await?;

        tx.commit().await?;

        let sagas: anyhow::Result<Vec<SagaState>> =
            rows.into_iter().map(|r| r.try_into()).collect();

        Ok((sagas?, total))
    }

    /// 未完了Sagaを検索する。一度に大量のSagaをロードしないようLIMITで件数を制限する。
    /// CRIT-005 注意: このメソッドは起動時リカバリ目的のため全テナント横断で実行する。
    /// tenant_id セッション変数を設定しないので RLS は NULL 比較になり全行がフィルタされる。
    /// 将来的には管理者ロール（RLS バイパス）の DB 接続を使用する実装に移行すること。
    async fn find_incomplete(&self) -> anyhow::Result<Vec<SagaState>> {
        // 全テナント横断アクセスのため一時的に RLS をセッション変数で回避する
        // app.current_tenant_id を空文字列に設定すると NULL 比較となりフィルタされるため、
        // 代わりに FORCE RLS を無効化したサービスロールを使用する想定だが
        // 現実装では pool 接続ユーザーが既に RLS バイパス権限を持つ DB ロールを使用していることを前提とする
        let rows = sqlx::query_as::<_, SagaStateRow>(
            r#"
            SELECT id, workflow_name, current_step, status, payload, correlation_id, initiated_by, error_message, tenant_id, created_at, updated_at
            FROM saga.saga_states
            WHERE status IN ('STARTED', 'RUNNING', 'COMPENSATING')
            ORDER BY created_at
            LIMIT 100
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.try_into()).collect()
    }
}

/// SagaStateRow はDB行からのマッピング用。
#[derive(sqlx::FromRow)]
struct SagaStateRow {
    id: Uuid,
    workflow_name: String,
    current_step: i32,
    status: String,
    payload: Option<serde_json::Value>,
    correlation_id: Option<String>,
    initiated_by: Option<String>,
    error_message: Option<String>,
    tenant_id: String,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl TryFrom<SagaStateRow> for SagaState {
    type Error = anyhow::Error;

    fn try_from(row: SagaStateRow) -> anyhow::Result<Self> {
        Ok(SagaState {
            saga_id: row.id,
            workflow_name: row.workflow_name,
            current_step: row.current_step,
            status: SagaStatus::from_str_value(&row.status)?,
            payload: row.payload.unwrap_or(serde_json::Value::Null),
            correlation_id: row.correlation_id,
            initiated_by: row.initiated_by,
            error_message: row.error_message,
            tenant_id: row.tenant_id,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

/// StepLogRow はDB行からのマッピング用。
#[derive(sqlx::FromRow)]
struct StepLogRow {
    id: Uuid,
    saga_id: Uuid,
    step_index: i32,
    step_name: String,
    action: String,
    status: String,
    request_payload: Option<serde_json::Value>,
    response_payload: Option<serde_json::Value>,
    error_message: Option<String>,
    started_at: chrono::DateTime<chrono::Utc>,
    completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl TryFrom<StepLogRow> for SagaStepLog {
    type Error = anyhow::Error;

    fn try_from(row: StepLogRow) -> anyhow::Result<Self> {
        let action = match row.action.as_str() {
            "EXECUTE" => StepAction::Execute,
            "COMPENSATE" => StepAction::Compensate,
            other => anyhow::bail!("invalid step action: {}", other),
        };
        let status = match row.status.as_str() {
            "SUCCESS" => StepStatus::Success,
            "FAILED" => StepStatus::Failed,
            "TIMEOUT" => StepStatus::Timeout,
            "SKIPPED" => StepStatus::Skipped,
            other => anyhow::bail!("invalid step status: {}", other),
        };

        Ok(SagaStepLog {
            id: row.id,
            saga_id: row.saga_id,
            step_index: row.step_index,
            step_name: row.step_name,
            action,
            status,
            request_payload: row.request_payload,
            response_payload: row.response_payload,
            error_message: row.error_message,
            started_at: row.started_at,
            completed_at: row.completed_at,
        })
    }
}
