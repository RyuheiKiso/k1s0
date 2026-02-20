use std::sync::Arc;

use tracing::{error, info, warn};
use uuid::Uuid;

use crate::domain::entity::saga_state::{SagaState, SagaStatus};
use crate::domain::entity::saga_step_log::SagaStepLog;
use crate::domain::entity::workflow::{RetryConfig, WorkflowDefinition, WorkflowStep};
use crate::domain::repository::SagaRepository;
use crate::infrastructure::grpc_caller::GrpcStepCaller;
use crate::infrastructure::kafka_producer::SagaEventPublisher;

/// ExecuteSagaUseCase はSagaの実行・補償ロジックを担う。
pub struct ExecuteSagaUseCase {
    saga_repo: Arc<dyn SagaRepository>,
    caller: Arc<dyn GrpcStepCaller>,
    publisher: Option<Arc<dyn SagaEventPublisher>>,
}

impl ExecuteSagaUseCase {
    pub fn new(
        saga_repo: Arc<dyn SagaRepository>,
        caller: Arc<dyn GrpcStepCaller>,
        publisher: Option<Arc<dyn SagaEventPublisher>>,
    ) -> Self {
        Self {
            saga_repo,
            caller,
            publisher,
        }
    }

    /// Sagaを実行する。ステータスに応じてforward_executeまたはcompensateを実行する。
    pub async fn run(&self, saga_id: Uuid, workflow: &WorkflowDefinition) -> anyhow::Result<()> {
        let state = self
            .saga_repo
            .find_by_id(saga_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("saga not found: {}", saga_id))?;

        if state.is_terminal() {
            info!(saga_id = %saga_id, status = %state.status, "saga already in terminal state");
            return Ok(());
        }

        let mut state = state;
        match state.status {
            SagaStatus::Started | SagaStatus::Running => {
                self.forward_execute(&mut state, workflow).await?;
            }
            SagaStatus::Compensating => {
                self.compensate(&mut state, workflow).await?;
            }
            _ => {
                info!(saga_id = %saga_id, status = %state.status, "saga in unexpected state");
            }
        }

        Ok(())
    }

    /// 前方実行：current_stepから最終ステップまで順次実行する。
    async fn forward_execute(
        &self,
        state: &mut SagaState,
        workflow: &WorkflowDefinition,
    ) -> anyhow::Result<()> {
        // Set status to RUNNING
        state.status = SagaStatus::Running;
        self.saga_repo
            .update_status(state.saga_id, &state.status, None)
            .await?;

        self.publish_event(state, "SAGA_RUNNING").await;

        let total_steps = workflow.steps.len();

        while (state.current_step as usize) < total_steps {
            let step_idx = state.current_step as usize;
            let step = &workflow.steps[step_idx];

            info!(
                saga_id = %state.saga_id,
                step = step_idx,
                step_name = %step.name,
                "executing step"
            );

            let mut step_log = SagaStepLog::new_execute(
                state.saga_id,
                state.current_step,
                step.name.clone(),
                Some(state.payload.clone()),
            );

            match self.execute_step_with_retry(step, &state.payload).await {
                Ok(response) => {
                    step_log.mark_success(Some(response));
                    state.advance_step();
                    self.saga_repo
                        .update_with_step_log(state, &step_log)
                        .await?;

                    info!(
                        saga_id = %state.saga_id,
                        step = step_idx,
                        step_name = %step.name,
                        "step completed successfully"
                    );
                }
                Err(e) => {
                    let error_msg = format!("step '{}' failed: {}", step.name, e);
                    step_log.mark_failed(error_msg.clone());
                    state.start_compensation(error_msg.clone());
                    self.saga_repo
                        .update_with_step_log(state, &step_log)
                        .await?;

                    error!(
                        saga_id = %state.saga_id,
                        step = step_idx,
                        step_name = %step.name,
                        error = %e,
                        "step failed, starting compensation"
                    );

                    self.publish_event(state, "SAGA_COMPENSATING").await;
                    self.compensate(state, workflow).await?;
                    return Ok(());
                }
            }
        }

        // All steps completed successfully
        state.complete();
        self.saga_repo
            .update_status(state.saga_id, &state.status, None)
            .await?;

        self.publish_event(state, "SAGA_COMPLETED").await;

        info!(saga_id = %state.saga_id, "saga completed successfully");
        Ok(())
    }

    /// リトライ付きでステップを実行する。
    async fn execute_step_with_retry(
        &self,
        step: &WorkflowStep,
        payload: &serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        let retry_config = step.retry.clone().unwrap_or(RetryConfig {
            max_attempts: 3,
            backoff: "exponential".to_string(),
            initial_interval_ms: 1000,
        });

        let timeout_duration = std::time::Duration::from_secs(step.timeout_secs);

        for attempt in 0..retry_config.max_attempts {
            if attempt > 0 {
                let delay = retry_config.delay_for_attempt(attempt - 1);
                info!(
                    step_name = %step.name,
                    attempt = attempt + 1,
                    delay_ms = delay.as_millis() as u64,
                    "retrying step"
                );
                tokio::time::sleep(delay).await;
            }

            match tokio::time::timeout(
                timeout_duration,
                self.caller.call_step(&step.service, &step.method, payload),
            )
            .await
            {
                Ok(Ok(response)) => return Ok(response),
                Ok(Err(e)) => {
                    warn!(
                        step_name = %step.name,
                        attempt = attempt + 1,
                        error = %e,
                        "step call failed"
                    );
                    if attempt + 1 >= retry_config.max_attempts {
                        return Err(e);
                    }
                }
                Err(_) => {
                    warn!(
                        step_name = %step.name,
                        attempt = attempt + 1,
                        timeout_secs = step.timeout_secs,
                        "step timed out"
                    );
                    if attempt + 1 >= retry_config.max_attempts {
                        return Err(anyhow::anyhow!(
                            "step '{}' timed out after {} attempts",
                            step.name,
                            retry_config.max_attempts
                        ));
                    }
                }
            }
        }

        Err(anyhow::anyhow!(
            "step '{}' failed after {} attempts",
            step.name,
            retry_config.max_attempts
        ))
    }

    /// 補償処理：逆順でcompensateメソッドを呼び出す。失敗しても継続（best-effort）。
    async fn compensate(
        &self,
        state: &mut SagaState,
        workflow: &WorkflowDefinition,
    ) -> anyhow::Result<()> {
        let comp_start = (state.current_step as usize).min(workflow.steps.len());

        info!(
            saga_id = %state.saga_id,
            from_step = comp_start.saturating_sub(1),
            "starting compensation"
        );

        // Compensate from the step before current (the last successfully executed step)
        // down to step 0
        for step_idx in (0..comp_start).rev() {
            let step = &workflow.steps[step_idx];

            let compensate_method = match &step.compensate {
                Some(method) => method,
                None => {
                    info!(
                        saga_id = %state.saga_id,
                        step = step_idx,
                        step_name = %step.name,
                        "no compensate method defined, skipping"
                    );
                    let mut step_log = SagaStepLog::new_compensate(
                        state.saga_id,
                        step_idx as i32,
                        step.name.clone(),
                        None,
                    );
                    step_log.status = crate::domain::entity::saga_step_log::StepStatus::Skipped;
                    step_log.completed_at = Some(chrono::Utc::now());
                    // Best effort: ignore repo errors during compensation logging
                    let _ = self.saga_repo.update_with_step_log(state, &step_log).await;
                    continue;
                }
            };

            info!(
                saga_id = %state.saga_id,
                step = step_idx,
                step_name = %step.name,
                compensate_method = %compensate_method,
                "compensating step"
            );

            let mut step_log = SagaStepLog::new_compensate(
                state.saga_id,
                step_idx as i32,
                step.name.clone(),
                Some(state.payload.clone()),
            );

            match self
                .caller
                .call_step(&step.service, compensate_method, &state.payload)
                .await
            {
                Ok(response) => {
                    step_log.mark_success(Some(response));
                    info!(
                        saga_id = %state.saga_id,
                        step = step_idx,
                        "compensation step succeeded"
                    );
                }
                Err(e) => {
                    step_log.mark_failed(format!("compensation failed: {}", e));
                    warn!(
                        saga_id = %state.saga_id,
                        step = step_idx,
                        error = %e,
                        "compensation step failed (best-effort, continuing)"
                    );
                }
            }

            // Best effort: ignore repo errors during compensation logging
            let _ = self.saga_repo.update_with_step_log(state, &step_log).await;
        }

        // Mark saga as FAILED after compensation
        let original_error = state.error_message.clone().unwrap_or_default();
        state.fail(original_error);
        self.saga_repo
            .update_status(state.saga_id, &state.status, state.error_message.clone())
            .await?;

        self.publish_event(state, "SAGA_FAILED").await;

        info!(saga_id = %state.saga_id, "compensation completed, saga marked as failed");
        Ok(())
    }

    /// イベントを発行する（オプショナル、失敗時はログのみ）。
    async fn publish_event(&self, state: &SagaState, event_type: &str) {
        if let Some(ref publisher) = self.publisher {
            if let Err(e) = publisher
                .publish_saga_event(&state.saga_id.to_string(), event_type, &state.payload)
                .await
            {
                warn!(
                    saga_id = %state.saga_id,
                    event_type = event_type,
                    error = %e,
                    "failed to publish saga event"
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::saga_state::SagaState;
    use crate::domain::entity::workflow::WorkflowDefinition;
    use crate::domain::repository::saga_repository::MockSagaRepository;
    use crate::infrastructure::grpc_caller::MockGrpcStepCaller;

    fn make_workflow() -> WorkflowDefinition {
        WorkflowDefinition::from_yaml(
            r#"
name: test-workflow
steps:
  - name: step-1
    service: svc-a
    method: SvcA.Do
    compensate: SvcA.Undo
    timeout_secs: 5
    retry:
      max_attempts: 1
      initial_interval_ms: 100
  - name: step-2
    service: svc-b
    method: SvcB.Do
    compensate: SvcB.Undo
    timeout_secs: 5
    retry:
      max_attempts: 1
      initial_interval_ms: 100
"#,
        )
        .unwrap()
    }

    fn make_saga() -> SagaState {
        SagaState::new(
            "test-workflow".to_string(),
            serde_json::json!({"key": "value"}),
            None,
            None,
        )
    }

    #[tokio::test]
    async fn test_successful_execution() {
        let saga = make_saga();
        let saga_id = saga.saga_id;
        let workflow = make_workflow();

        let mut mock_repo = MockSagaRepository::new();
        let saga_clone = saga.clone();
        mock_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(saga_clone.clone())));
        mock_repo.expect_update_status().returning(|_, _, _| Ok(()));
        mock_repo
            .expect_update_with_step_log()
            .returning(|_, _| Ok(()));

        let mut mock_caller = MockGrpcStepCaller::new();
        mock_caller
            .expect_call_step()
            .returning(|_, _, _| Ok(serde_json::json!({"ok": true})));

        let uc = ExecuteSagaUseCase::new(Arc::new(mock_repo), Arc::new(mock_caller), None);

        let result = uc.run(saga_id, &workflow).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_step_failure_triggers_compensation() {
        let saga = make_saga();
        let saga_id = saga.saga_id;
        let workflow = make_workflow();

        let mut mock_repo = MockSagaRepository::new();
        let saga_clone = saga.clone();
        // Return a saga that has already completed step 0
        mock_repo.expect_find_by_id().returning(move |_| {
            let mut s = saga_clone.clone();
            s.advance_step(); // step 0 done, now at step 1
            Ok(Some(s))
        });
        mock_repo.expect_update_status().returning(|_, _, _| Ok(()));
        mock_repo
            .expect_update_with_step_log()
            .returning(|_, _| Ok(()));

        let mut mock_caller = MockGrpcStepCaller::new();
        let call_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let call_count_clone = call_count.clone();
        mock_caller
            .expect_call_step()
            .returning(move |_, method, _| {
                let count = call_count_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                if count == 0 {
                    // step-2 execute fails
                    Err(anyhow::anyhow!("service unavailable"))
                } else {
                    // compensation calls succeed
                    Ok(serde_json::json!({"compensated": true}))
                }
            });

        let uc = ExecuteSagaUseCase::new(Arc::new(mock_repo), Arc::new(mock_caller), None);

        let result = uc.run(saga_id, &workflow).await;
        assert!(result.is_ok()); // compensation completes without error
    }

    #[tokio::test]
    async fn test_terminal_state_skipped() {
        let mut saga = make_saga();
        saga.complete(); // already completed
        let saga_id = saga.saga_id;
        let workflow = make_workflow();

        let mut mock_repo = MockSagaRepository::new();
        let saga_clone = saga.clone();
        mock_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(saga_clone.clone())));

        let mock_caller = MockGrpcStepCaller::new();

        let uc = ExecuteSagaUseCase::new(Arc::new(mock_repo), Arc::new(mock_caller), None);

        let result = uc.run(saga_id, &workflow).await;
        assert!(result.is_ok());
    }
}
