// 実行PostgreSQLリポジトリ
// sqlxを使用してPostgreSQLからExecutionを読み書きする

use std::sync::Arc;

use async_trait::async_trait;
use sqlx::PgPool;

use crate::domain::entity::{Execution, ExecutionStatus, ExecutionStep};
use crate::domain::repository::ExecutionRepository;

/// `ExecutionPostgresRepository` `はPostgreSQLベースの実行リポジトリ実装`
pub struct ExecutionPostgresRepository {
    /// データベースコネクションプール
    pool: Arc<PgPool>,
}

impl ExecutionPostgresRepository {
    /// `新しいExecutionPostgresRepositoryを生成する`
    #[must_use]
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ExecutionRepository for ExecutionPostgresRepository {
    /// IDで実行を検索する
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<Execution>> {
        let row = sqlx::query_as::<_, ExecutionRow>(
            "SELECT id, agent_id, session_id, tenant_id, input, output, status, steps, created_at, updated_at FROM executions WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(self.pool.as_ref())
        .await?;

        Ok(row.map(std::convert::Into::into))
    }

    /// 実行をUPSERTで保存する
    async fn save(&self, execution: &Execution) -> anyhow::Result<()> {
        let steps_json = serde_json::to_value(&execution.steps)?;
        sqlx::query(
            "INSERT INTO executions (id, agent_id, session_id, tenant_id, input, output, status, steps, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) \
             ON CONFLICT (id) DO UPDATE SET output = $6, status = $7, steps = $8, updated_at = $10"
        )
        .bind(&execution.id)
        .bind(&execution.agent_id)
        .bind(&execution.session_id)
        .bind(&execution.tenant_id)
        .bind(&execution.input)
        .bind(&execution.output)
        .bind(execution.status.to_string())
        .bind(&steps_json)
        .bind(execution.created_at)
        .bind(execution.updated_at)
        .execute(self.pool.as_ref())
        .await?;

        Ok(())
    }

    /// エージェントIDで実行履歴を取得する
    async fn find_by_agent(&self, agent_id: &str) -> anyhow::Result<Vec<Execution>> {
        let rows = sqlx::query_as::<_, ExecutionRow>(
            "SELECT id, agent_id, session_id, tenant_id, input, output, status, steps, created_at, updated_at FROM executions WHERE agent_id = $1 ORDER BY created_at DESC"
        )
        .bind(agent_id)
        .fetch_all(self.pool.as_ref())
        .await?;

        Ok(rows.into_iter().map(std::convert::Into::into).collect())
    }
}

/// データベース行からExecutionへのマッピング用構造体
#[derive(sqlx::FromRow)]
struct ExecutionRow {
    id: String,
    agent_id: String,
    session_id: String,
    tenant_id: String,
    input: String,
    output: Option<String>,
    status: String,
    steps: serde_json::Value,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<ExecutionRow> for Execution {
    fn from(row: ExecutionRow) -> Self {
        // HIGH-001 監査対応: ステータス文字列をEnumに変換する（同一ボディのアームをORで結合）
        let status = match row.status.as_str() {
            "running" => ExecutionStatus::Running,
            "completed" => ExecutionStatus::Completed,
            "failed" => ExecutionStatus::Failed,
            "cancelled" => ExecutionStatus::Cancelled,
            _ => ExecutionStatus::Pending,
        };
        let steps: Vec<ExecutionStep> = serde_json::from_value(row.steps).unwrap_or_default();
        Self {
            id: row.id,
            agent_id: row.agent_id,
            session_id: row.session_id,
            tenant_id: row.tenant_id,
            input: row.input,
            output: row.output,
            status,
            steps,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
