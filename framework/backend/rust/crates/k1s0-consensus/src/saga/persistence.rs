//! Saga 永続化（PostgreSQL）。

use chrono::Utc;
use sqlx::PgPool;

use crate::error::ConsensusResult;
use crate::saga::{SagaInstance, SagaStatus};

/// Saga インスタンスを作成する。
pub async fn insert_saga_instance(
    pool: &PgPool,
    saga_id: &str,
    saga_name: &str,
    context: &serde_json::Value,
) -> ConsensusResult<()> {
    sqlx::query(
        r"
        INSERT INTO fw_m_saga_instance (saga_id, saga_name, status, current_step, context, created_at, updated_at)
        VALUES ($1, $2, 'RUNNING', 0, $3, $4, $4)
        ",
    )
    .bind(saga_id)
    .bind(saga_name)
    .bind(context)
    .bind(Utc::now())
    .execute(pool)
    .await?;

    Ok(())
}

/// Saga のステータスを更新する。
pub async fn update_saga_status(
    pool: &PgPool,
    saga_id: &str,
    status: SagaStatus,
    error_message: Option<&str>,
) -> ConsensusResult<()> {
    sqlx::query(
        r"
        UPDATE fw_m_saga_instance
        SET status = $1, error_message = $2, updated_at = $3
        WHERE saga_id = $4
        ",
    )
    .bind(status.to_string())
    .bind(error_message)
    .bind(Utc::now())
    .bind(saga_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Saga ステップの実行結果を記録する。
pub async fn insert_step_record(
    pool: &PgPool,
    saga_id: &str,
    step_name: &str,
    step_index: i32,
    status: &str,
    input: &serde_json::Value,
    output: Option<&serde_json::Value>,
    error_message: Option<&str>,
) -> ConsensusResult<()> {
    sqlx::query(
        r"
        INSERT INTO fw_m_saga_step (saga_id, step_name, step_index, status, input, output, error_message, executed_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ",
    )
    .bind(saga_id)
    .bind(step_name)
    .bind(step_index)
    .bind(status)
    .bind(input)
    .bind(output)
    .bind(error_message)
    .bind(Utc::now())
    .execute(pool)
    .await?;

    Ok(())
}

/// デッドレターキューの Saga を取得する。
pub async fn query_dead_letters(pool: &PgPool, limit: u32) -> ConsensusResult<Vec<SagaInstance>> {
    let rows = sqlx::query_as::<_, (String, String, String, i32, serde_json::Value, Option<String>, chrono::DateTime<Utc>, chrono::DateTime<Utc>)>(
        r"
        SELECT saga_id, saga_name, status, current_step, context, error_message, created_at, updated_at
        FROM fw_m_saga_instance
        WHERE status = 'DEAD_LETTER'
        ORDER BY updated_at DESC
        LIMIT $1
        ",
    )
    .bind(i64::from(limit))
    .fetch_all(pool)
    .await?;

    let instances = rows
        .into_iter()
        .map(
            |(saga_id, saga_name, status_str, current_step, context, error_message, created_at, updated_at)| {
                let status = match status_str.as_str() {
                    "DEAD_LETTER" => SagaStatus::DeadLetter,
                    "COMPLETED" => SagaStatus::Completed,
                    "COMPENSATED" => SagaStatus::Compensated,
                    "COMPENSATING" => SagaStatus::Compensating,
                    "TIMED_OUT" => SagaStatus::TimedOut,
                    _ => SagaStatus::Running,
                };
                SagaInstance {
                    saga_id,
                    saga_name,
                    status,
                    current_step,
                    context,
                    error_message,
                    created_at,
                    updated_at,
                }
            },
        )
        .collect();

    Ok(instances)
}
