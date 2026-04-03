use std::sync::Arc;

use sqlx::PgPool;

use crate::domain::entity::workflow_instance::WorkflowInstance;
use crate::domain::entity::workflow_task::WorkflowTask;
use crate::domain::repository::WorkflowDefinitionRepository;
use crate::domain::repository::WorkflowInstanceRepository;
use crate::domain::repository::WorkflowTaskRepository;
use crate::domain::service::WorkflowDomainService;
use crate::infrastructure::kafka_producer::WorkflowEventPublisher;
// B-MEDIUM-02 監査対応: postgres_support を adapter/repository レイヤーからインポートする
use crate::adapter::repository::postgres_support::{insert_instance_tx, insert_task_tx};
use tracing::error;

// RUST-CRIT-001 対応: テナント分離のため tenant_id フィールドを追加する
#[derive(Debug, Clone)]
pub struct StartInstanceInput {
    pub tenant_id: String,
    pub workflow_id: String,
    pub title: String,
    pub initiator_id: String,
    pub context: serde_json::Value,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct StartInstanceOutput {
    pub instance: WorkflowInstance,
    pub first_task: Option<WorkflowTask>,
}

#[derive(Debug, thiserror::Error)]
pub enum StartInstanceError {
    #[error("workflow not found: {0}")]
    WorkflowNotFound(String),

    #[error("workflow disabled: {0}")]
    WorkflowDisabled(String),

    #[error("workflow has no steps: {0}")]
    NoSteps(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct StartInstanceUseCase {
    definition_repo: Arc<dyn WorkflowDefinitionRepository>,
    instance_repo: Arc<dyn WorkflowInstanceRepository>,
    task_repo: Arc<dyn WorkflowTaskRepository>,
    event_publisher: Arc<dyn WorkflowEventPublisher>,
    pool: Option<Arc<PgPool>>,
}

impl StartInstanceUseCase {
    pub fn new(
        definition_repo: Arc<dyn WorkflowDefinitionRepository>,
        instance_repo: Arc<dyn WorkflowInstanceRepository>,
        task_repo: Arc<dyn WorkflowTaskRepository>,
        event_publisher: Arc<dyn WorkflowEventPublisher>,
    ) -> Self {
        Self {
            definition_repo,
            instance_repo,
            task_repo,
            event_publisher,
            pool: None,
        }
    }

    pub fn with_pool(
        definition_repo: Arc<dyn WorkflowDefinitionRepository>,
        instance_repo: Arc<dyn WorkflowInstanceRepository>,
        task_repo: Arc<dyn WorkflowTaskRepository>,
        event_publisher: Arc<dyn WorkflowEventPublisher>,
        pool: Arc<PgPool>,
    ) -> Self {
        Self {
            definition_repo,
            instance_repo,
            task_repo,
            event_publisher,
            pool: Some(pool),
        }
    }

    pub async fn execute(
        &self,
        input: &StartInstanceInput,
    ) -> Result<StartInstanceOutput, StartInstanceError> {
        // テナント分離: tenant_id を渡してRLSによるフィルタリングを有効化する
        let definition = self
            .definition_repo
            .find_by_id(&input.tenant_id, &input.workflow_id)
            .await
            .map_err(|e| StartInstanceError::Internal(e.to_string()))?
            .ok_or_else(|| StartInstanceError::WorkflowNotFound(input.workflow_id.clone()))?;

        if !WorkflowDomainService::can_start_workflow(definition.enabled) {
            return Err(StartInstanceError::WorkflowDisabled(
                input.workflow_id.clone(),
            ));
        }

        let first_step = definition
            .first_step()
            .ok_or_else(|| StartInstanceError::NoSteps(input.workflow_id.clone()))?;

        // LOW-09 対応: 非標準の "inst_" プレフィックス付き ID から標準 UUID v4 文字列に変更する。
        // DB の UUID 型との互換性を確保するため、to_string() で標準ハイフン区切り形式（RFC 4122）を使用する。
        // 注意: 既存データの instance_id フォーマットは変更しない（新規作成のみ適用）。
        let instance_id = uuid::Uuid::new_v4().to_string();
        let instance = WorkflowInstance::new(
            instance_id.clone(),
            definition.id.clone(),
            definition.name.clone(),
            input.title.clone(),
            input.initiator_id.clone(),
            Some(first_step.step_id.clone()),
            input.context.clone(),
        );

        // LOW-09: task_id も同様に標準 UUID v4 文字列に変更する
        let task_id = uuid::Uuid::new_v4().to_string();
        let due_at = WorkflowDomainService::task_due_at(first_step.timeout_hours);
        let task = WorkflowTask::new(
            task_id,
            instance_id,
            first_step.step_id.clone(),
            first_step.name.clone(),
            None,
            due_at,
        );

        if let Some(pool) = &self.pool {
            // テナント分離: トランザクション内でtenant_idを渡してRLSを有効化する
            let mut tx = pool
                .begin()
                .await
                .map_err(|e| StartInstanceError::Internal(e.to_string()))?;
            insert_instance_tx(&mut tx, &instance, &input.tenant_id)
                .await
                .map_err(|e| StartInstanceError::Internal(e.to_string()))?;
            insert_task_tx(&mut tx, &task, &input.tenant_id)
                .await
                .map_err(|e| StartInstanceError::Internal(e.to_string()))?;
            tx.commit()
                .await
                .map_err(|e| StartInstanceError::Internal(e.to_string()))?;
        } else {
            self.instance_repo
                .create(&input.tenant_id, &instance)
                .await
                .map_err(|e| StartInstanceError::Internal(e.to_string()))?;

            self.task_repo
                .create(&input.tenant_id, &task)
                .await
                .map_err(|e| StartInstanceError::Internal(e.to_string()))?;
        }

        if let Err(err) = self
            .event_publisher
            .publish_instance_started(&instance)
            .await
        {
            // Kafka 配信失敗はデータ損失リスクがあるため error! レベルで記録する
            error!(
                instance_id = %instance.id,
                workflow_id = %instance.workflow_id,
                error = %err,
                "workflow インスタンス開始イベントの Kafka 配信に失敗しました"
            );
        }

        Ok(StartInstanceOutput {
            instance,
            first_task: Some(task),
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::entity::workflow_definition::WorkflowDefinition;
    use crate::domain::entity::workflow_step::WorkflowStep;
    use crate::domain::repository::workflow_definition_repository::MockWorkflowDefinitionRepository;
    use crate::domain::repository::workflow_instance_repository::MockWorkflowInstanceRepository;
    use crate::domain::repository::workflow_task_repository::MockWorkflowTaskRepository;
    use crate::infrastructure::kafka_producer::MockWorkflowEventPublisher;

    fn sample_definition() -> WorkflowDefinition {
        WorkflowDefinition::new(
            "wf_001".to_string(),
            "purchase-approval".to_string(),
            "".to_string(),
            true,
            vec![WorkflowStep::new(
                "step-1".to_string(),
                "Approval".to_string(),
                "human_task".to_string(),
                Some("manager".to_string()),
                Some(48),
                Some("end".to_string()),
                Some("end".to_string()),
            )],
        )
    }

    #[tokio::test]
    async fn success() {
        let mut def_mock = MockWorkflowDefinitionRepository::new();
        let mut inst_mock = MockWorkflowInstanceRepository::new();
        let mut task_mock = MockWorkflowTaskRepository::new();

        def_mock
            .expect_find_by_id()
            .returning(|_, _| Ok(Some(sample_definition())));
        inst_mock.expect_create().returning(|_, _| Ok(()));
        task_mock.expect_create().returning(|_, _| Ok(()));
        let mut publisher = MockWorkflowEventPublisher::new();
        publisher
            .expect_publish_instance_started()
            .times(1)
            .returning(|_| Ok(()));

        let uc = StartInstanceUseCase::new(
            Arc::new(def_mock),
            Arc::new(inst_mock),
            Arc::new(task_mock),
            Arc::new(publisher),
        );
        let input = StartInstanceInput {
            tenant_id: "test-tenant".to_string(),
            workflow_id: "wf_001".to_string(),
            title: "PC Purchase".to_string(),
            initiator_id: "user-001".to_string(),
            context: serde_json::json!({"item": "laptop"}),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.instance.status, "running");
        assert!(output.first_task.is_some());
    }

    #[tokio::test]
    async fn workflow_not_found() {
        let mut def_mock = MockWorkflowDefinitionRepository::new();
        let inst_mock = MockWorkflowInstanceRepository::new();
        let task_mock = MockWorkflowTaskRepository::new();
        let publisher = MockWorkflowEventPublisher::new();

        def_mock.expect_find_by_id().returning(|_, _| Ok(None));

        let uc = StartInstanceUseCase::new(
            Arc::new(def_mock),
            Arc::new(inst_mock),
            Arc::new(task_mock),
            Arc::new(publisher),
        );
        let input = StartInstanceInput {
            tenant_id: "test-tenant".to_string(),
            workflow_id: "wf_missing".to_string(),
            title: "test".to_string(),
            initiator_id: "user-001".to_string(),
            context: serde_json::json!({}),
        };
        let result = uc.execute(&input).await;
        assert!(matches!(
            result.unwrap_err(),
            StartInstanceError::WorkflowNotFound(_)
        ));
    }

    #[tokio::test]
    async fn workflow_disabled() {
        let mut def_mock = MockWorkflowDefinitionRepository::new();
        let inst_mock = MockWorkflowInstanceRepository::new();
        let task_mock = MockWorkflowTaskRepository::new();
        let publisher = MockWorkflowEventPublisher::new();

        let mut def = sample_definition();
        def.enabled = false;
        def_mock
            .expect_find_by_id()
            .returning(move |_, _| Ok(Some(def.clone())));

        let uc = StartInstanceUseCase::new(
            Arc::new(def_mock),
            Arc::new(inst_mock),
            Arc::new(task_mock),
            Arc::new(publisher),
        );
        let input = StartInstanceInput {
            tenant_id: "test-tenant".to_string(),
            workflow_id: "wf_001".to_string(),
            title: "test".to_string(),
            initiator_id: "user-001".to_string(),
            context: serde_json::json!({}),
        };
        let result = uc.execute(&input).await;
        assert!(matches!(
            result.unwrap_err(),
            StartInstanceError::WorkflowDisabled(_)
        ));
    }

    #[tokio::test]
    async fn internal_error() {
        let mut def_mock = MockWorkflowDefinitionRepository::new();
        let inst_mock = MockWorkflowInstanceRepository::new();
        let task_mock = MockWorkflowTaskRepository::new();
        let publisher = MockWorkflowEventPublisher::new();

        def_mock
            .expect_find_by_id()
            .returning(|_, _| Err(anyhow::anyhow!("db error")));

        let uc = StartInstanceUseCase::new(
            Arc::new(def_mock),
            Arc::new(inst_mock),
            Arc::new(task_mock),
            Arc::new(publisher),
        );
        let input = StartInstanceInput {
            tenant_id: "test-tenant".to_string(),
            workflow_id: "wf_001".to_string(),
            title: "test".to_string(),
            initiator_id: "user-001".to_string(),
            context: serde_json::json!({}),
        };
        let result = uc.execute(&input).await;
        assert!(matches!(
            result.unwrap_err(),
            StartInstanceError::Internal(_)
        ));
    }

    #[tokio::test]
    async fn publish_failure_does_not_fail_usecase() {
        let mut def_mock = MockWorkflowDefinitionRepository::new();
        let mut inst_mock = MockWorkflowInstanceRepository::new();
        let mut task_mock = MockWorkflowTaskRepository::new();
        let mut publisher = MockWorkflowEventPublisher::new();

        def_mock
            .expect_find_by_id()
            .returning(|_, _| Ok(Some(sample_definition())));
        inst_mock.expect_create().returning(|_, _| Ok(()));
        task_mock.expect_create().returning(|_, _| Ok(()));
        publisher
            .expect_publish_instance_started()
            .times(1)
            .returning(|_| Err(anyhow::anyhow!("kafka unavailable")));

        let uc = StartInstanceUseCase::new(
            Arc::new(def_mock),
            Arc::new(inst_mock),
            Arc::new(task_mock),
            Arc::new(publisher),
        );
        let result = uc
            .execute(&StartInstanceInput {
                tenant_id: "test-tenant".to_string(),
                workflow_id: "wf_001".to_string(),
                title: "PC Purchase".to_string(),
                initiator_id: "user-001".to_string(),
                context: serde_json::json!({"item": "laptop"}),
            })
            .await;

        assert!(result.is_ok());
    }
}
