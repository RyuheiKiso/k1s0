//! Saga オーケストレータ実装。

use uuid::Uuid;

use crate::error::ConsensusResult;
use crate::saga::definition::RetryPolicy;
use crate::saga::{compensator, SagaDefinition, SagaResult, SagaStatus, SagaStepError};

/// インメモリ Saga オーケストレータ。
///
/// テストやシングルプロセス環境向け。
/// 永続化が必要な場合は `postgres` feature の `PersistentSagaOrchestrator` を使用する。
pub struct InMemorySagaOrchestrator;

impl InMemorySagaOrchestrator {
    /// Saga を実行する。
    ///
    /// # Errors
    ///
    /// ステップの実行や補償に失敗した場合にエラーを返す。
    pub async fn execute(
        &self,
        definition: &SagaDefinition,
        initial_context: serde_json::Value,
    ) -> ConsensusResult<SagaResult> {
        let saga_id = Uuid::new_v4().to_string();
        let mut context = initial_context;
        let mut completed_outputs: Vec<serde_json::Value> = Vec::new();

        tracing::info!(saga_id = %saga_id, saga_name = %definition.name, "starting saga");
        super::metrics::saga_started().inc();

        for (i, entry) in definition.steps.iter().enumerate() {
            let step = &entry.step;
            let retry_policy = definition.retry_policy_for(i);

            tracing::info!(
                saga_id = %saga_id,
                step_name = step.name(),
                step_index = i,
                "executing step"
            );

            match execute_with_retry(step.as_ref(), &context, retry_policy).await {
                Ok(output) => {
                    completed_outputs.push(output.clone());
                    context = output;
                }
                Err(reason) => {
                    tracing::error!(
                        saga_id = %saga_id,
                        step_name = step.name(),
                        step_index = i,
                        error = %reason,
                        "step failed, starting compensation"
                    );
                    super::metrics::saga_failed().inc();

                    // 補償処理
                    match compensator::compensate(definition, &saga_id, &completed_outputs, i).await
                    {
                        Ok(()) => {
                            return Ok(SagaResult {
                                saga_id,
                                status: SagaStatus::Compensated,
                                output: None,
                                error: Some(reason),
                            });
                        }
                        Err(comp_err) => {
                            tracing::error!(
                                saga_id = %saga_id,
                                error = %comp_err,
                                "compensation failed, moving to dead letter"
                            );
                            super::metrics::saga_dead_letter().inc();

                            return Ok(SagaResult {
                                saga_id,
                                status: SagaStatus::DeadLetter,
                                output: None,
                                error: Some(comp_err.to_string()),
                            });
                        }
                    }
                }
            }
        }

        tracing::info!(saga_id = %saga_id, saga_name = %definition.name, "saga completed");
        super::metrics::saga_completed().inc();

        Ok(SagaResult {
            saga_id,
            status: SagaStatus::Completed,
            output: Some(context),
            error: None,
        })
    }
}

/// リトライポリシーに従ってステップを実行する。
async fn execute_with_retry(
    step: &dyn crate::saga::SagaStep,
    context: &serde_json::Value,
    retry_policy: &RetryPolicy,
) -> Result<serde_json::Value, String> {
    let mut last_error = String::new();

    for attempt in 0..=retry_policy.max_retries {
        if attempt > 0 {
            let delay = retry_policy.backoff.delay_ms(attempt - 1);
            tracing::debug!(
                step_name = step.name(),
                attempt,
                delay_ms = delay,
                "retrying step"
            );
            tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
        }

        match step.execute(context).await {
            Ok(output) => return Ok(output),
            Err(SagaStepError::NonRetryable(reason)) => {
                return Err(reason);
            }
            Err(SagaStepError::Retryable(reason)) => {
                last_error = reason;
            }
        }
    }

    Err(last_error)
}

/// 永続化付き Saga オーケストレータ（PostgreSQL バックエンド）。
#[cfg(feature = "postgres")]
pub struct PersistentSagaOrchestrator {
    pool: sqlx::PgPool,
    inner: InMemorySagaOrchestrator,
}

#[cfg(feature = "postgres")]
impl PersistentSagaOrchestrator {
    /// 新しい `PersistentSagaOrchestrator` を作成する。
    #[must_use]
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self {
            pool,
            inner: InMemorySagaOrchestrator,
        }
    }

    /// Saga を実行し、結果を永続化する。
    ///
    /// # Errors
    ///
    /// ステップの実行、補償、またはデータベース操作に失敗した場合にエラーを返す。
    pub async fn execute(
        &self,
        definition: &SagaDefinition,
        initial_context: serde_json::Value,
    ) -> ConsensusResult<SagaResult> {
        // インスタンスを作成
        let saga_id = Uuid::new_v4().to_string();
        super::persistence::insert_saga_instance(
            &self.pool,
            &saga_id,
            &definition.name,
            &initial_context,
        )
        .await?;

        // 実行
        let result = self.inner.execute(definition, initial_context).await?;

        // 結果を永続化
        super::persistence::update_saga_status(
            &self.pool,
            &result.saga_id,
            result.status,
            result.error.as_deref(),
        )
        .await?;

        Ok(result)
    }

    /// デッドレターキューの Saga 一覧を取得する。
    ///
    /// # Errors
    ///
    /// データベース操作に失敗した場合にエラーを返す。
    pub async fn dead_letters(&self, limit: u32) -> ConsensusResult<Vec<SagaInstance>> {
        super::persistence::query_dead_letters(&self.pool, limit).await
    }
}
