use std::sync::Arc;

use sqlx::PgPool;

use crate::domain::entity::workflow_task::WorkflowTask;
use crate::domain::repository::WorkflowDefinitionRepository;
use crate::domain::repository::WorkflowInstanceRepository;
use crate::domain::repository::WorkflowTaskRepository;
use crate::domain::service::WorkflowDomainService;
use crate::infrastructure::kafka_producer::WorkflowEventPublisher;
// B-MEDIUM-02 監査対応: postgres_support を adapter/repository レイヤーからインポートする
use crate::adapter::repository::postgres_support::{
    insert_task_tx, update_instance_tx, update_task_tx,
};
use tracing::warn;

// RUST-CRIT-001 対応: テナント分離のため tenant_id フィールドを追加する
#[derive(Debug, Clone)]
pub struct RejectTaskInput {
    pub tenant_id: String,
    pub task_id: String,
    pub actor_id: String,
    pub comment: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RejectTaskOutput {
    pub task: WorkflowTask,
    pub next_task: Option<WorkflowTask>,
    pub instance_status: String,
}

#[derive(Debug, thiserror::Error)]
pub enum RejectTaskError {
    #[error("task not found: {0}")]
    TaskNotFound(String),

    #[error("invalid task status for rejection: {0}")]
    InvalidStatus(String),

    #[error("instance not found: {0}")]
    InstanceNotFound(String),

    #[error("workflow definition not found: {0}")]
    DefinitionNotFound(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct RejectTaskUseCase {
    task_repo: Arc<dyn WorkflowTaskRepository>,
    instance_repo: Arc<dyn WorkflowInstanceRepository>,
    definition_repo: Arc<dyn WorkflowDefinitionRepository>,
    event_publisher: Arc<dyn WorkflowEventPublisher>,
    pool: Option<Arc<PgPool>>,
}

impl RejectTaskUseCase {
    pub fn new(
        task_repo: Arc<dyn WorkflowTaskRepository>,
        instance_repo: Arc<dyn WorkflowInstanceRepository>,
        definition_repo: Arc<dyn WorkflowDefinitionRepository>,
        event_publisher: Arc<dyn WorkflowEventPublisher>,
    ) -> Self {
        Self {
            task_repo,
            instance_repo,
            definition_repo,
            event_publisher,
            pool: None,
        }
    }

    pub fn with_pool(
        task_repo: Arc<dyn WorkflowTaskRepository>,
        instance_repo: Arc<dyn WorkflowInstanceRepository>,
        definition_repo: Arc<dyn WorkflowDefinitionRepository>,
        event_publisher: Arc<dyn WorkflowEventPublisher>,
        pool: Arc<PgPool>,
    ) -> Self {
        Self {
            task_repo,
            instance_repo,
            definition_repo,
            event_publisher,
            pool: Some(pool),
        }
    }

    // タスク否認ロジックはトランザクション・状態機械・イベント発行を含むため行数が多い
    #[allow(clippy::too_many_lines)]
    pub async fn execute(
        &self,
        input: &RejectTaskInput,
    ) -> Result<RejectTaskOutput, RejectTaskError> {
        // テナント分離: tenant_id を渡してRLSによるフィルタリングを有効化する
        let mut task = self
            .task_repo
            .find_by_id(&input.tenant_id, &input.task_id)
            .await
            .map_err(|e| RejectTaskError::Internal(e.to_string()))?
            .ok_or_else(|| RejectTaskError::TaskNotFound(input.task_id.clone()))?;

        if !task.is_decidable() {
            return Err(RejectTaskError::InvalidStatus(task.status.clone()));
        }

        task.reject(input.actor_id.clone(), input.comment.clone());

        let mut instance = self
            .instance_repo
            .find_by_id(&input.tenant_id, &task.instance_id)
            .await
            .map_err(|e| RejectTaskError::Internal(e.to_string()))?
            .ok_or_else(|| RejectTaskError::InstanceNotFound(task.instance_id.clone()))?;

        let definition = self
            .definition_repo
            .find_by_id(&input.tenant_id, &instance.workflow_id)
            .await
            .map_err(|e| RejectTaskError::Internal(e.to_string()))?
            .ok_or_else(|| RejectTaskError::DefinitionNotFound(instance.workflow_id.clone()))?;

        let next_step_id = WorkflowDomainService::next_step_on_reject(&definition, &task.step_id);

        let mut next_task = None;

        if WorkflowDomainService::is_terminal_step(next_step_id.as_deref()) {
            instance.fail();
        } else {
            match next_step_id.as_deref() {
                Some(next_id) => {
                    if let Some(next_step) = definition.find_step(next_id) {
                        instance.current_step_id = Some(next_id.to_string());

                        let new_task_id = format!("task_{}", uuid::Uuid::new_v4().simple());
                        let due_at = WorkflowDomainService::task_due_at(next_step.timeout_hours);
                        let new_task = WorkflowTask::new(
                            new_task_id,
                            instance.id.clone(),
                            next_step.step_id.clone(),
                            next_step.name.clone(),
                            None,
                            due_at,
                        );
                        next_task = Some(new_task);
                    } else {
                        instance.fail();
                    }
                }
                None => {
                    instance.fail();
                }
            }
        }

        if let Some(pool) = &self.pool {
            // テナント分離: トランザクション内でtenant_idを渡してRLSを有効化する
            let mut tx = pool
                .begin()
                .await
                .map_err(|e| RejectTaskError::Internal(e.to_string()))?;
            update_task_tx(&mut tx, &task, &input.tenant_id)
                .await
                .map_err(|e| RejectTaskError::Internal(e.to_string()))?;
            update_instance_tx(&mut tx, &instance, &input.tenant_id)
                .await
                .map_err(|e| RejectTaskError::Internal(e.to_string()))?;
            if let Some(next_task_ref) = next_task.as_ref() {
                insert_task_tx(&mut tx, next_task_ref, &input.tenant_id)
                    .await
                    .map_err(|e| RejectTaskError::Internal(e.to_string()))?;
            }
            tx.commit()
                .await
                .map_err(|e| RejectTaskError::Internal(e.to_string()))?;
        } else {
            self.task_repo
                .update(&input.tenant_id, &task)
                .await
                .map_err(|e| RejectTaskError::Internal(e.to_string()))?;
            self.instance_repo
                .update(&input.tenant_id, &instance)
                .await
                .map_err(|e| RejectTaskError::Internal(e.to_string()))?;
            if let Some(next_task_ref) = next_task.as_ref() {
                self.task_repo
                    .create(&input.tenant_id, next_task_ref)
                    .await
                    .map_err(|e| RejectTaskError::Internal(e.to_string()))?;
            }
        }

        if let Err(err) = self.event_publisher.publish_task_completed(&task).await {
            warn!(
                task_id = %task.id,
                instance_id = %task.instance_id,
                error = %err,
                "failed to publish workflow task completed event"
            );
        }

        Ok(RejectTaskOutput {
            task,
            next_task,
            instance_status: instance.status,
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::entity::workflow_definition::WorkflowDefinition;
    use crate::domain::entity::workflow_instance::WorkflowInstance;
    use crate::domain::entity::workflow_step::WorkflowStep;
    use crate::domain::repository::workflow_definition_repository::MockWorkflowDefinitionRepository;
    use crate::domain::repository::workflow_instance_repository::MockWorkflowInstanceRepository;
    use crate::domain::repository::workflow_task_repository::MockWorkflowTaskRepository;
    use crate::infrastructure::kafka_producer::MockWorkflowEventPublisher;

    fn two_step_definition() -> WorkflowDefinition {
        WorkflowDefinition::new(
            "wf_001".to_string(),
            "test".to_string(),
            "".to_string(),
            true,
            vec![
                WorkflowStep::new(
                    "step-1".to_string(),
                    "First".to_string(),
                    "human_task".to_string(),
                    Some("manager".to_string()),
                    Some(48),
                    Some("step-2".to_string()),
                    Some("end".to_string()),
                ),
                WorkflowStep::new(
                    "step-2".to_string(),
                    "Second".to_string(),
                    "human_task".to_string(),
                    Some("finance".to_string()),
                    Some(72),
                    Some("end".to_string()),
                    Some("step-1".to_string()),
                ),
            ],
        )
    }

    fn running_instance() -> WorkflowInstance {
        WorkflowInstance::new(
            "inst_001".to_string(),
            "wf_001".to_string(),
            "test".to_string(),
            "title".to_string(),
            "user-001".to_string(),
            Some("step-1".to_string()),
            serde_json::json!({}),
        )
    }

    fn assigned_task() -> WorkflowTask {
        WorkflowTask::new(
            "task_001".to_string(),
            "inst_001".to_string(),
            "step-1".to_string(),
            "First".to_string(),
            Some("user-002".to_string()),
            None,
        )
    }

    #[tokio::test]
    async fn success_fails_instance_on_end() {
        let mut task_mock = MockWorkflowTaskRepository::new();
        let mut inst_mock = MockWorkflowInstanceRepository::new();
        let mut def_mock = MockWorkflowDefinitionRepository::new();

        task_mock
            .expect_find_by_id()
            .returning(|_, _| Ok(Some(assigned_task())));
        task_mock.expect_update().returning(|_, _| Ok(()));
        inst_mock
            .expect_find_by_id()
            .returning(|_, _| Ok(Some(running_instance())));
        inst_mock.expect_update().returning(|_, _| Ok(()));
        def_mock
            .expect_find_by_id()
            .returning(|_, _| Ok(Some(two_step_definition())));
        let mut publisher = MockWorkflowEventPublisher::new();
        publisher
            .expect_publish_task_completed()
            .times(1)
            .returning(|_| Ok(()));

        let uc = RejectTaskUseCase::new(
            Arc::new(task_mock),
            Arc::new(inst_mock),
            Arc::new(def_mock),
            Arc::new(publisher),
        );
        let input = RejectTaskInput {
            tenant_id: "test-tenant".to_string(),
            task_id: "task_001".to_string(),
            actor_id: "user-002".to_string(),
            comment: Some("Rejected".to_string()),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.task.status, "rejected");
        assert!(output.next_task.is_none());
        assert_eq!(output.instance_status, "failed");
    }

    #[tokio::test]
    async fn success_remands_to_previous_step() {
        let mut task_mock = MockWorkflowTaskRepository::new();
        let mut inst_mock = MockWorkflowInstanceRepository::new();
        let mut def_mock = MockWorkflowDefinitionRepository::new();

        let mut task = assigned_task();
        task.step_id = "step-2".to_string();
        task_mock
            .expect_find_by_id()
            .returning(move |_, _| Ok(Some(task.clone())));
        task_mock.expect_update().returning(|_, _| Ok(()));
        task_mock.expect_create().returning(|_, _| Ok(()));

        let mut inst = running_instance();
        inst.current_step_id = Some("step-2".to_string());
        inst_mock
            .expect_find_by_id()
            .returning(move |_, _| Ok(Some(inst.clone())));
        inst_mock.expect_update().returning(|_, _| Ok(()));

        def_mock
            .expect_find_by_id()
            .returning(|_, _| Ok(Some(two_step_definition())));
        let mut publisher = MockWorkflowEventPublisher::new();
        publisher
            .expect_publish_task_completed()
            .times(1)
            .returning(|_| Ok(()));

        let uc = RejectTaskUseCase::new(
            Arc::new(task_mock),
            Arc::new(inst_mock),
            Arc::new(def_mock),
            Arc::new(publisher),
        );
        let input = RejectTaskInput {
            tenant_id: "test-tenant".to_string(),
            task_id: "task_001".to_string(),
            actor_id: "user-003".to_string(),
            comment: Some("Budget exceeded".to_string()),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.next_task.is_some());
        assert_eq!(output.instance_status, "running");
    }

    #[tokio::test]
    async fn task_not_found() {
        let mut task_mock = MockWorkflowTaskRepository::new();
        let inst_mock = MockWorkflowInstanceRepository::new();
        let def_mock = MockWorkflowDefinitionRepository::new();
        let publisher = MockWorkflowEventPublisher::new();

        task_mock.expect_find_by_id().returning(|_, _| Ok(None));

        let uc = RejectTaskUseCase::new(
            Arc::new(task_mock),
            Arc::new(inst_mock),
            Arc::new(def_mock),
            Arc::new(publisher),
        );
        let input = RejectTaskInput {
            tenant_id: "test-tenant".to_string(),
            task_id: "task_missing".to_string(),
            actor_id: "user-002".to_string(),
            comment: None,
        };
        let result = uc.execute(&input).await;
        assert!(matches!(
            result.unwrap_err(),
            RejectTaskError::TaskNotFound(_)
        ));
    }

    #[tokio::test]
    async fn invalid_status() {
        let mut task_mock = MockWorkflowTaskRepository::new();
        let inst_mock = MockWorkflowInstanceRepository::new();
        let def_mock = MockWorkflowDefinitionRepository::new();
        let publisher = MockWorkflowEventPublisher::new();

        let mut task = assigned_task();
        task.approve("prev".to_string(), None);
        task_mock
            .expect_find_by_id()
            .returning(move |_, _| Ok(Some(task.clone())));

        let uc = RejectTaskUseCase::new(
            Arc::new(task_mock),
            Arc::new(inst_mock),
            Arc::new(def_mock),
            Arc::new(publisher),
        );
        let input = RejectTaskInput {
            tenant_id: "test-tenant".to_string(),
            task_id: "task_001".to_string(),
            actor_id: "user-002".to_string(),
            comment: None,
        };
        let result = uc.execute(&input).await;
        assert!(matches!(
            result.unwrap_err(),
            RejectTaskError::InvalidStatus(_)
        ));
    }

    #[tokio::test]
    async fn publish_failure_does_not_fail_usecase() {
        let mut task_mock = MockWorkflowTaskRepository::new();
        let mut inst_mock = MockWorkflowInstanceRepository::new();
        let mut def_mock = MockWorkflowDefinitionRepository::new();
        let mut publisher = MockWorkflowEventPublisher::new();

        task_mock
            .expect_find_by_id()
            .returning(|_, _| Ok(Some(assigned_task())));
        task_mock.expect_update().returning(|_, _| Ok(()));
        inst_mock
            .expect_find_by_id()
            .returning(|_, _| Ok(Some(running_instance())));
        inst_mock.expect_update().returning(|_, _| Ok(()));
        def_mock
            .expect_find_by_id()
            .returning(|_, _| Ok(Some(two_step_definition())));
        publisher
            .expect_publish_task_completed()
            .times(1)
            .returning(|_| Err(anyhow::anyhow!("kafka unavailable")));

        let uc = RejectTaskUseCase::new(
            Arc::new(task_mock),
            Arc::new(inst_mock),
            Arc::new(def_mock),
            Arc::new(publisher),
        );
        let result = uc
            .execute(&RejectTaskInput {
                tenant_id: "test-tenant".to_string(),
                task_id: "task_001".to_string(),
                actor_id: "user-002".to_string(),
                comment: Some("Rejected".to_string()),
            })
            .await;

        assert!(result.is_ok());
    }
}
