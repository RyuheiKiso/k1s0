use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use tokio::sync::RwLock;

use k1s0_workflow_server::domain::entity::workflow_definition::WorkflowDefinition;
use k1s0_workflow_server::domain::entity::workflow_instance::WorkflowInstance;
use k1s0_workflow_server::domain::entity::workflow_step::WorkflowStep;
use k1s0_workflow_server::domain::entity::workflow_task::WorkflowTask;
use k1s0_workflow_server::domain::repository::{
    WorkflowDefinitionRepository, WorkflowInstanceRepository, WorkflowTaskRepository,
};
use k1s0_workflow_server::infrastructure::kafka_producer::WorkflowEventPublisher;
use k1s0_workflow_server::infrastructure::notification_request_producer::NotificationRequestPublisher;

use k1s0_workflow_server::usecase::approve_task::{ApproveTaskInput, ApproveTaskUseCase};
use k1s0_workflow_server::usecase::cancel_instance::{CancelInstanceInput, CancelInstanceUseCase};
use k1s0_workflow_server::usecase::check_overdue_tasks::CheckOverdueTasksUseCase;
use k1s0_workflow_server::usecase::create_workflow::{CreateWorkflowInput, CreateWorkflowUseCase};
use k1s0_workflow_server::usecase::delete_workflow::{DeleteWorkflowInput, DeleteWorkflowUseCase};
use k1s0_workflow_server::usecase::get_instance::{GetInstanceInput, GetInstanceUseCase};
use k1s0_workflow_server::usecase::get_workflow::{GetWorkflowInput, GetWorkflowUseCase};
use k1s0_workflow_server::usecase::list_instances::{ListInstancesInput, ListInstancesUseCase};
use k1s0_workflow_server::usecase::list_tasks::{ListTasksInput, ListTasksUseCase};
use k1s0_workflow_server::usecase::list_workflows::{ListWorkflowsInput, ListWorkflowsUseCase};
use k1s0_workflow_server::usecase::reassign_task::{ReassignTaskInput, ReassignTaskUseCase};
use k1s0_workflow_server::usecase::reject_task::{RejectTaskInput, RejectTaskUseCase};
use k1s0_workflow_server::usecase::start_instance::{StartInstanceInput, StartInstanceUseCase};
use k1s0_workflow_server::usecase::update_workflow::{UpdateWorkflowInput, UpdateWorkflowUseCase};

// ============================================================
// Stub implementations
// ============================================================

/// In-memory stub for WorkflowDefinitionRepository.
struct StubDefinitionRepo {
    definitions: RwLock<Vec<WorkflowDefinition>>,
    should_error: bool,
}

impl StubDefinitionRepo {
    fn new() -> Self {
        Self {
            definitions: RwLock::new(Vec::new()),
            should_error: false,
        }
    }

    fn with_definitions(defs: Vec<WorkflowDefinition>) -> Self {
        Self {
            definitions: RwLock::new(defs),
            should_error: false,
        }
    }

    fn with_error() -> Self {
        Self {
            definitions: RwLock::new(Vec::new()),
            should_error: true,
        }
    }
}

#[async_trait]
impl WorkflowDefinitionRepository for StubDefinitionRepo {
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<WorkflowDefinition>> {
        if self.should_error {
            return Err(anyhow::anyhow!("stub db error"));
        }
        let defs = self.definitions.read().await;
        Ok(defs.iter().find(|d| d.id == id).cloned())
    }

    async fn find_by_name(&self, name: &str) -> anyhow::Result<Option<WorkflowDefinition>> {
        if self.should_error {
            return Err(anyhow::anyhow!("stub db error"));
        }
        let defs = self.definitions.read().await;
        Ok(defs.iter().find(|d| d.name == name).cloned())
    }

    async fn find_all(
        &self,
        enabled_only: bool,
        _page: u32,
        _page_size: u32,
    ) -> anyhow::Result<(Vec<WorkflowDefinition>, u64)> {
        if self.should_error {
            return Err(anyhow::anyhow!("stub db error"));
        }
        let defs = self.definitions.read().await;
        let filtered: Vec<_> = if enabled_only {
            defs.iter().filter(|d| d.enabled).cloned().collect()
        } else {
            defs.clone()
        };
        let total = filtered.len() as u64;
        Ok((filtered, total))
    }

    async fn create(&self, definition: &WorkflowDefinition) -> anyhow::Result<()> {
        if self.should_error {
            return Err(anyhow::anyhow!("stub db error"));
        }
        self.definitions.write().await.push(definition.clone());
        Ok(())
    }

    async fn update(&self, definition: &WorkflowDefinition) -> anyhow::Result<()> {
        if self.should_error {
            return Err(anyhow::anyhow!("stub db error"));
        }
        let mut defs = self.definitions.write().await;
        if let Some(pos) = defs.iter().position(|d| d.id == definition.id) {
            defs[pos] = definition.clone();
        }
        Ok(())
    }

    async fn delete(&self, id: &str) -> anyhow::Result<bool> {
        if self.should_error {
            return Err(anyhow::anyhow!("stub db error"));
        }
        let mut defs = self.definitions.write().await;
        let len_before = defs.len();
        defs.retain(|d| d.id != id);
        Ok(defs.len() < len_before)
    }
}

/// In-memory stub for WorkflowInstanceRepository.
struct StubInstanceRepo {
    instances: RwLock<Vec<WorkflowInstance>>,
    should_error: bool,
}

impl StubInstanceRepo {
    fn new() -> Self {
        Self {
            instances: RwLock::new(Vec::new()),
            should_error: false,
        }
    }

    fn with_instances(instances: Vec<WorkflowInstance>) -> Self {
        Self {
            instances: RwLock::new(instances),
            should_error: false,
        }
    }

    fn with_error() -> Self {
        Self {
            instances: RwLock::new(Vec::new()),
            should_error: true,
        }
    }
}

#[async_trait]
impl WorkflowInstanceRepository for StubInstanceRepo {
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<WorkflowInstance>> {
        if self.should_error {
            return Err(anyhow::anyhow!("stub db error"));
        }
        let instances = self.instances.read().await;
        Ok(instances.iter().find(|i| i.id == id).cloned())
    }

    async fn find_all(
        &self,
        status: Option<String>,
        workflow_id: Option<String>,
        initiator_id: Option<String>,
        _page: u32,
        _page_size: u32,
    ) -> anyhow::Result<(Vec<WorkflowInstance>, u64)> {
        if self.should_error {
            return Err(anyhow::anyhow!("stub db error"));
        }
        let instances = self.instances.read().await;
        let filtered: Vec<_> = instances
            .iter()
            .filter(|i| {
                if let Some(ref s) = status {
                    if i.status != *s {
                        return false;
                    }
                }
                if let Some(ref wid) = workflow_id {
                    if i.workflow_id != *wid {
                        return false;
                    }
                }
                if let Some(ref iid) = initiator_id {
                    if i.initiator_id != *iid {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();
        let total = filtered.len() as u64;
        Ok((filtered, total))
    }

    async fn create(&self, instance: &WorkflowInstance) -> anyhow::Result<()> {
        if self.should_error {
            return Err(anyhow::anyhow!("stub db error"));
        }
        self.instances.write().await.push(instance.clone());
        Ok(())
    }

    async fn update(&self, instance: &WorkflowInstance) -> anyhow::Result<()> {
        if self.should_error {
            return Err(anyhow::anyhow!("stub db error"));
        }
        let mut instances = self.instances.write().await;
        if let Some(pos) = instances.iter().position(|i| i.id == instance.id) {
            instances[pos] = instance.clone();
        }
        Ok(())
    }
}

/// In-memory stub for WorkflowTaskRepository.
struct StubTaskRepo {
    tasks: RwLock<Vec<WorkflowTask>>,
    should_error: bool,
}

impl StubTaskRepo {
    fn new() -> Self {
        Self {
            tasks: RwLock::new(Vec::new()),
            should_error: false,
        }
    }

    fn with_tasks(tasks: Vec<WorkflowTask>) -> Self {
        Self {
            tasks: RwLock::new(tasks),
            should_error: false,
        }
    }

    fn with_error() -> Self {
        Self {
            tasks: RwLock::new(Vec::new()),
            should_error: true,
        }
    }
}

#[async_trait]
impl WorkflowTaskRepository for StubTaskRepo {
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<WorkflowTask>> {
        if self.should_error {
            return Err(anyhow::anyhow!("stub db error"));
        }
        let tasks = self.tasks.read().await;
        Ok(tasks.iter().find(|t| t.id == id).cloned())
    }

    async fn find_all(
        &self,
        assignee_id: Option<String>,
        status: Option<String>,
        instance_id: Option<String>,
        overdue_only: bool,
        _page: u32,
        _page_size: u32,
    ) -> anyhow::Result<(Vec<WorkflowTask>, u64)> {
        if self.should_error {
            return Err(anyhow::anyhow!("stub db error"));
        }
        let tasks = self.tasks.read().await;
        let filtered: Vec<_> = tasks
            .iter()
            .filter(|t| {
                if let Some(ref aid) = assignee_id {
                    if t.assignee_id.as_deref() != Some(aid.as_str()) {
                        return false;
                    }
                }
                if let Some(ref s) = status {
                    if t.status != *s {
                        return false;
                    }
                }
                if let Some(ref iid) = instance_id {
                    if t.instance_id != *iid {
                        return false;
                    }
                }
                if overdue_only && !t.is_overdue() {
                    return false;
                }
                true
            })
            .cloned()
            .collect();
        let total = filtered.len() as u64;
        Ok((filtered, total))
    }

    async fn find_overdue(&self) -> anyhow::Result<Vec<WorkflowTask>> {
        if self.should_error {
            return Err(anyhow::anyhow!("stub db error"));
        }
        let tasks = self.tasks.read().await;
        Ok(tasks.iter().filter(|t| t.is_overdue()).cloned().collect())
    }

    async fn create(&self, task: &WorkflowTask) -> anyhow::Result<()> {
        if self.should_error {
            return Err(anyhow::anyhow!("stub db error"));
        }
        self.tasks.write().await.push(task.clone());
        Ok(())
    }

    async fn update(&self, task: &WorkflowTask) -> anyhow::Result<()> {
        if self.should_error {
            return Err(anyhow::anyhow!("stub db error"));
        }
        let mut tasks = self.tasks.write().await;
        if let Some(pos) = tasks.iter().position(|t| t.id == task.id) {
            tasks[pos] = task.clone();
        }
        Ok(())
    }
}

/// Stub for WorkflowEventPublisher (noop, always succeeds).
struct StubEventPublisher;

#[async_trait]
impl WorkflowEventPublisher for StubEventPublisher {
    async fn publish_instance_started(&self, _instance: &WorkflowInstance) -> anyhow::Result<()> {
        Ok(())
    }
    async fn publish_task_completed(&self, _task: &WorkflowTask) -> anyhow::Result<()> {
        Ok(())
    }
    async fn close(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

/// Stub for WorkflowEventPublisher that always fails.
struct FailingEventPublisher;

#[async_trait]
impl WorkflowEventPublisher for FailingEventPublisher {
    async fn publish_instance_started(&self, _instance: &WorkflowInstance) -> anyhow::Result<()> {
        Err(anyhow::anyhow!("kafka unavailable"))
    }
    async fn publish_task_completed(&self, _task: &WorkflowTask) -> anyhow::Result<()> {
        Err(anyhow::anyhow!("kafka unavailable"))
    }
    async fn close(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

/// Stub for NotificationRequestPublisher (noop).
struct StubNotificationPublisher;

#[async_trait]
impl NotificationRequestPublisher for StubNotificationPublisher {
    async fn publish_task_overdue(&self, _task: &WorkflowTask) -> anyhow::Result<()> {
        Ok(())
    }
    async fn close(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

/// Stub for NotificationRequestPublisher that always fails.
struct FailingNotificationPublisher;

#[async_trait]
impl NotificationRequestPublisher for FailingNotificationPublisher {
    async fn publish_task_overdue(&self, _task: &WorkflowTask) -> anyhow::Result<()> {
        Err(anyhow::anyhow!("kafka unavailable"))
    }
    async fn close(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

// ============================================================
// Test helpers
// ============================================================

fn sample_steps() -> Vec<WorkflowStep> {
    vec![WorkflowStep::new(
        "step-1".to_string(),
        "Approval".to_string(),
        "human_task".to_string(),
        Some("manager".to_string()),
        Some(48),
        Some("end".to_string()),
        Some("end".to_string()),
    )]
}

fn two_step_definition() -> WorkflowDefinition {
    WorkflowDefinition::new(
        "wf_001".to_string(),
        "purchase-approval".to_string(),
        "Purchase approval flow".to_string(),
        true,
        vec![
            WorkflowStep::new(
                "step-1".to_string(),
                "Manager Approval".to_string(),
                "human_task".to_string(),
                Some("manager".to_string()),
                Some(48),
                Some("step-2".to_string()),
                Some("end".to_string()),
            ),
            WorkflowStep::new(
                "step-2".to_string(),
                "Finance Approval".to_string(),
                "human_task".to_string(),
                Some("finance".to_string()),
                Some(72),
                Some("end".to_string()),
                Some("step-1".to_string()),
            ),
        ],
    )
}

fn single_step_definition() -> WorkflowDefinition {
    WorkflowDefinition::new(
        "wf_002".to_string(),
        "simple-approval".to_string(),
        "Simple one-step approval".to_string(),
        true,
        sample_steps(),
    )
}

fn disabled_definition() -> WorkflowDefinition {
    let mut def = single_step_definition();
    def.id = "wf_disabled".to_string();
    def.name = "disabled-workflow".to_string();
    def.enabled = false;
    def
}

fn empty_step_definition() -> WorkflowDefinition {
    WorkflowDefinition::new(
        "wf_empty".to_string(),
        "empty-workflow".to_string(),
        "No steps".to_string(),
        true,
        vec![],
    )
}

fn running_instance() -> WorkflowInstance {
    WorkflowInstance::new(
        "inst_001".to_string(),
        "wf_001".to_string(),
        "purchase-approval".to_string(),
        "PC Purchase".to_string(),
        "user-001".to_string(),
        Some("step-1".to_string()),
        serde_json::json!({"item": "laptop"}),
    )
}

fn completed_instance() -> WorkflowInstance {
    let mut inst = running_instance();
    inst.id = "inst_completed".to_string();
    inst.complete();
    inst
}

fn assigned_task() -> WorkflowTask {
    WorkflowTask::new(
        "task_001".to_string(),
        "inst_001".to_string(),
        "step-1".to_string(),
        "Manager Approval".to_string(),
        Some("user-002".to_string()),
        None,
    )
}

fn pending_task() -> WorkflowTask {
    WorkflowTask::new(
        "task_002".to_string(),
        "inst_001".to_string(),
        "step-1".to_string(),
        "Manager Approval".to_string(),
        None,
        None,
    )
}

fn overdue_task() -> WorkflowTask {
    let mut task = WorkflowTask::new(
        "task_overdue".to_string(),
        "inst_001".to_string(),
        "step-1".to_string(),
        "Approval".to_string(),
        Some("user-002".to_string()),
        Some(Utc::now() - chrono::Duration::hours(1)),
    );
    task.due_at = Some(Utc::now() - chrono::Duration::hours(1));
    task
}

// ============================================================
// CreateWorkflowUseCase tests
// ============================================================

mod create_workflow {
    use super::*;

    #[tokio::test]
    async fn success_creates_new_workflow() {
        let repo = Arc::new(StubDefinitionRepo::new());
        let uc = CreateWorkflowUseCase::new(repo.clone());

        let input = CreateWorkflowInput {
            name: "purchase-approval".to_string(),
            description: "Purchase flow".to_string(),
            enabled: true,
            steps: sample_steps(),
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let def = result.unwrap();
        assert_eq!(def.name, "purchase-approval");
        assert_eq!(def.description, "Purchase flow");
        assert_eq!(def.version, 1);
        assert!(def.enabled);
        assert_eq!(def.step_count(), 1);
        assert!(def.id.starts_with("wf_"));

        // Verify persisted
        let defs = repo.definitions.read().await;
        assert_eq!(defs.len(), 1);
        assert_eq!(defs[0].name, "purchase-approval");
    }

    #[tokio::test]
    async fn error_already_exists() {
        let existing = single_step_definition();
        let repo = Arc::new(StubDefinitionRepo::with_definitions(vec![existing]));
        let uc = CreateWorkflowUseCase::new(repo);

        let input = CreateWorkflowInput {
            name: "simple-approval".to_string(),
            description: "".to_string(),
            enabled: true,
            steps: sample_steps(),
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("already exists"));
    }

    #[tokio::test]
    async fn error_empty_name() {
        let repo = Arc::new(StubDefinitionRepo::new());
        let uc = CreateWorkflowUseCase::new(repo);

        let input = CreateWorkflowInput {
            name: "".to_string(),
            description: "".to_string(),
            enabled: true,
            steps: sample_steps(),
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("name is required"));
    }

    #[tokio::test]
    async fn error_no_steps() {
        let repo = Arc::new(StubDefinitionRepo::new());
        let uc = CreateWorkflowUseCase::new(repo);

        let input = CreateWorkflowInput {
            name: "test".to_string(),
            description: "".to_string(),
            enabled: true,
            steps: vec![],
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("at least one step"));
    }

    #[tokio::test]
    async fn error_internal_repo_failure() {
        let repo = Arc::new(StubDefinitionRepo::with_error());
        let uc = CreateWorkflowUseCase::new(repo);

        let input = CreateWorkflowInput {
            name: "test".to_string(),
            description: "".to_string(),
            enabled: true,
            steps: sample_steps(),
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("internal error"));
    }
}

// ============================================================
// GetWorkflowUseCase tests
// ============================================================

mod get_workflow {
    use super::*;

    #[tokio::test]
    async fn success_finds_workflow() {
        let def = single_step_definition();
        let repo = Arc::new(StubDefinitionRepo::with_definitions(vec![def]));
        let uc = GetWorkflowUseCase::new(repo);

        let input = GetWorkflowInput {
            id: "wf_002".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, "wf_002");
    }

    #[tokio::test]
    async fn error_not_found() {
        let repo = Arc::new(StubDefinitionRepo::new());
        let uc = GetWorkflowUseCase::new(repo);

        let input = GetWorkflowInput {
            id: "wf_nonexistent".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("not found"));
    }

    #[tokio::test]
    async fn error_internal() {
        let repo = Arc::new(StubDefinitionRepo::with_error());
        let uc = GetWorkflowUseCase::new(repo);

        let input = GetWorkflowInput {
            id: "wf_001".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("internal error"));
    }
}

// ============================================================
// UpdateWorkflowUseCase tests
// ============================================================

mod update_workflow {
    use super::*;

    #[tokio::test]
    async fn success_updates_name_and_enabled() {
        let def = single_step_definition();
        let repo = Arc::new(StubDefinitionRepo::with_definitions(vec![def]));
        let uc = UpdateWorkflowUseCase::new(repo.clone());

        let input = UpdateWorkflowInput {
            id: "wf_002".to_string(),
            name: Some("updated-name".to_string()),
            description: None,
            enabled: Some(false),
            steps: None,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let updated = result.unwrap();
        assert_eq!(updated.name, "updated-name");
        assert!(!updated.enabled);
        assert_eq!(updated.version, 2);

        // Verify persisted
        let defs = repo.definitions.read().await;
        assert_eq!(defs[0].name, "updated-name");
        assert_eq!(defs[0].version, 2);
    }

    #[tokio::test]
    async fn success_partial_update_preserves_other_fields() {
        let def = single_step_definition();
        let repo = Arc::new(StubDefinitionRepo::with_definitions(vec![def]));
        let uc = UpdateWorkflowUseCase::new(repo);

        let input = UpdateWorkflowInput {
            id: "wf_002".to_string(),
            name: None,
            description: Some("new description".to_string()),
            enabled: None,
            steps: None,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let updated = result.unwrap();
        assert_eq!(updated.name, "simple-approval"); // unchanged
        assert_eq!(updated.description, "new description");
        assert!(updated.enabled); // unchanged
    }

    #[tokio::test]
    async fn error_not_found() {
        let repo = Arc::new(StubDefinitionRepo::new());
        let uc = UpdateWorkflowUseCase::new(repo);

        let input = UpdateWorkflowInput {
            id: "wf_missing".to_string(),
            name: Some("x".to_string()),
            description: None,
            enabled: None,
            steps: None,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("not found"));
    }

    #[tokio::test]
    async fn error_internal() {
        let repo = Arc::new(StubDefinitionRepo::with_error());
        let uc = UpdateWorkflowUseCase::new(repo);

        let input = UpdateWorkflowInput {
            id: "wf_001".to_string(),
            name: None,
            description: None,
            enabled: None,
            steps: None,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("internal error"));
    }
}

// ============================================================
// DeleteWorkflowUseCase tests
// ============================================================

mod delete_workflow {
    use super::*;

    #[tokio::test]
    async fn success_deletes_workflow() {
        let def = single_step_definition();
        let repo = Arc::new(StubDefinitionRepo::with_definitions(vec![def]));
        let uc = DeleteWorkflowUseCase::new(repo.clone());

        let input = DeleteWorkflowInput {
            id: "wf_002".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let defs = repo.definitions.read().await;
        assert!(defs.is_empty());
    }

    #[tokio::test]
    async fn error_not_found() {
        let repo = Arc::new(StubDefinitionRepo::new());
        let uc = DeleteWorkflowUseCase::new(repo);

        let input = DeleteWorkflowInput {
            id: "wf_missing".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("not found"));
    }

    #[tokio::test]
    async fn error_internal() {
        let repo = Arc::new(StubDefinitionRepo::with_error());
        let uc = DeleteWorkflowUseCase::new(repo);

        let input = DeleteWorkflowInput {
            id: "wf_001".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("internal error"));
    }
}

// ============================================================
// ListWorkflowsUseCase tests
// ============================================================

mod list_workflows {
    use super::*;

    #[tokio::test]
    async fn success_returns_all() {
        let defs = vec![single_step_definition(), two_step_definition()];
        let repo = Arc::new(StubDefinitionRepo::with_definitions(defs));
        let uc = ListWorkflowsUseCase::new(repo);

        let input = ListWorkflowsInput {
            enabled_only: false,
            page: 1,
            page_size: 20,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.workflows.len(), 2);
        assert_eq!(output.total_count, 2);
    }

    #[tokio::test]
    async fn success_filters_enabled_only() {
        let defs = vec![single_step_definition(), disabled_definition()];
        let repo = Arc::new(StubDefinitionRepo::with_definitions(defs));
        let uc = ListWorkflowsUseCase::new(repo);

        let input = ListWorkflowsInput {
            enabled_only: true,
            page: 1,
            page_size: 20,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.workflows.len(), 1);
        assert_eq!(output.workflows[0].name, "simple-approval");
    }

    #[tokio::test]
    async fn success_empty_list() {
        let repo = Arc::new(StubDefinitionRepo::new());
        let uc = ListWorkflowsUseCase::new(repo);

        let input = ListWorkflowsInput {
            enabled_only: false,
            page: 1,
            page_size: 20,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.workflows.is_empty());
        assert_eq!(output.total_count, 0);
        assert!(!output.has_next);
    }

    #[tokio::test]
    async fn error_internal() {
        let repo = Arc::new(StubDefinitionRepo::with_error());
        let uc = ListWorkflowsUseCase::new(repo);

        let input = ListWorkflowsInput {
            enabled_only: false,
            page: 1,
            page_size: 20,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());
    }
}

// ============================================================
// StartInstanceUseCase tests
// ============================================================

mod start_instance {
    use super::*;

    #[tokio::test]
    async fn success_creates_instance_and_first_task() {
        let def = two_step_definition();
        let def_repo = Arc::new(StubDefinitionRepo::with_definitions(vec![def]));
        let inst_repo = Arc::new(StubInstanceRepo::new());
        let task_repo = Arc::new(StubTaskRepo::new());
        let publisher = Arc::new(StubEventPublisher);

        let uc = StartInstanceUseCase::new(
            def_repo,
            inst_repo.clone(),
            task_repo.clone(),
            publisher,
        );

        let input = StartInstanceInput {
            workflow_id: "wf_001".to_string(),
            title: "PC Purchase".to_string(),
            initiator_id: "user-001".to_string(),
            context: serde_json::json!({"item": "laptop", "amount": 1500}),
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.instance.status, "running");
        assert_eq!(output.instance.workflow_id, "wf_001");
        assert_eq!(output.instance.workflow_name, "purchase-approval");
        assert_eq!(output.instance.title, "PC Purchase");
        assert_eq!(output.instance.initiator_id, "user-001");
        assert_eq!(
            output.instance.current_step_id,
            Some("step-1".to_string())
        );
        assert!(output.instance.completed_at.is_none());
        assert!(output.first_task.is_some());

        let first_task = output.first_task.unwrap();
        assert_eq!(first_task.step_id, "step-1");
        assert_eq!(first_task.step_name, "Manager Approval");
        assert_eq!(first_task.status, "pending"); // no assignee set by usecase

        // Verify persisted
        let instances = inst_repo.instances.read().await;
        assert_eq!(instances.len(), 1);
        let tasks = task_repo.tasks.read().await;
        assert_eq!(tasks.len(), 1);
    }

    #[tokio::test]
    async fn error_workflow_not_found() {
        let def_repo = Arc::new(StubDefinitionRepo::new());
        let inst_repo = Arc::new(StubInstanceRepo::new());
        let task_repo = Arc::new(StubTaskRepo::new());
        let publisher = Arc::new(StubEventPublisher);

        let uc = StartInstanceUseCase::new(def_repo, inst_repo, task_repo, publisher);

        let input = StartInstanceInput {
            workflow_id: "wf_nonexistent".to_string(),
            title: "Test".to_string(),
            initiator_id: "user-001".to_string(),
            context: serde_json::json!({}),
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("not found"));
    }

    #[tokio::test]
    async fn error_workflow_disabled() {
        let def = disabled_definition();
        let def_repo = Arc::new(StubDefinitionRepo::with_definitions(vec![def]));
        let inst_repo = Arc::new(StubInstanceRepo::new());
        let task_repo = Arc::new(StubTaskRepo::new());
        let publisher = Arc::new(StubEventPublisher);

        let uc = StartInstanceUseCase::new(def_repo, inst_repo, task_repo, publisher);

        let input = StartInstanceInput {
            workflow_id: "wf_disabled".to_string(),
            title: "Test".to_string(),
            initiator_id: "user-001".to_string(),
            context: serde_json::json!({}),
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("disabled"));
    }

    #[tokio::test]
    async fn error_workflow_no_steps() {
        let def = empty_step_definition();
        let def_repo = Arc::new(StubDefinitionRepo::with_definitions(vec![def]));
        let inst_repo = Arc::new(StubInstanceRepo::new());
        let task_repo = Arc::new(StubTaskRepo::new());
        let publisher = Arc::new(StubEventPublisher);

        let uc = StartInstanceUseCase::new(def_repo, inst_repo, task_repo, publisher);

        let input = StartInstanceInput {
            workflow_id: "wf_empty".to_string(),
            title: "Test".to_string(),
            initiator_id: "user-001".to_string(),
            context: serde_json::json!({}),
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("no steps"));
    }

    #[tokio::test]
    async fn error_internal_repo_failure() {
        let def_repo = Arc::new(StubDefinitionRepo::with_error());
        let inst_repo = Arc::new(StubInstanceRepo::new());
        let task_repo = Arc::new(StubTaskRepo::new());
        let publisher = Arc::new(StubEventPublisher);

        let uc = StartInstanceUseCase::new(def_repo, inst_repo, task_repo, publisher);

        let input = StartInstanceInput {
            workflow_id: "wf_001".to_string(),
            title: "Test".to_string(),
            initiator_id: "user-001".to_string(),
            context: serde_json::json!({}),
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("internal error"));
    }

    #[tokio::test]
    async fn publish_failure_does_not_fail_usecase() {
        let def = two_step_definition();
        let def_repo = Arc::new(StubDefinitionRepo::with_definitions(vec![def]));
        let inst_repo = Arc::new(StubInstanceRepo::new());
        let task_repo = Arc::new(StubTaskRepo::new());
        let publisher = Arc::new(FailingEventPublisher);

        let uc = StartInstanceUseCase::new(def_repo, inst_repo, task_repo, publisher);

        let input = StartInstanceInput {
            workflow_id: "wf_001".to_string(),
            title: "PC Purchase".to_string(),
            initiator_id: "user-001".to_string(),
            context: serde_json::json!({}),
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());
    }
}

// ============================================================
// GetInstanceUseCase tests
// ============================================================

mod get_instance {
    use super::*;

    #[tokio::test]
    async fn success_finds_instance() {
        let inst = running_instance();
        let repo = Arc::new(StubInstanceRepo::with_instances(vec![inst]));
        let uc = GetInstanceUseCase::new(repo);

        let input = GetInstanceInput {
            id: "inst_001".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let instance = result.unwrap();
        assert_eq!(instance.id, "inst_001");
        assert_eq!(instance.status, "running");
    }

    #[tokio::test]
    async fn error_not_found() {
        let repo = Arc::new(StubInstanceRepo::new());
        let uc = GetInstanceUseCase::new(repo);

        let input = GetInstanceInput {
            id: "inst_missing".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("not found"));
    }

    #[tokio::test]
    async fn error_internal() {
        let repo = Arc::new(StubInstanceRepo::with_error());
        let uc = GetInstanceUseCase::new(repo);

        let input = GetInstanceInput {
            id: "inst_001".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("internal error"));
    }
}

// ============================================================
// ListInstancesUseCase tests
// ============================================================

mod list_instances {
    use super::*;

    #[tokio::test]
    async fn success_returns_all() {
        let instances = vec![running_instance(), completed_instance()];
        let repo = Arc::new(StubInstanceRepo::with_instances(instances));
        let uc = ListInstancesUseCase::new(repo);

        let input = ListInstancesInput {
            status: None,
            workflow_id: None,
            initiator_id: None,
            page: 1,
            page_size: 20,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.instances.len(), 2);
        assert_eq!(output.total_count, 2);
    }

    #[tokio::test]
    async fn success_filters_by_status() {
        let instances = vec![running_instance(), completed_instance()];
        let repo = Arc::new(StubInstanceRepo::with_instances(instances));
        let uc = ListInstancesUseCase::new(repo);

        let input = ListInstancesInput {
            status: Some("running".to_string()),
            workflow_id: None,
            initiator_id: None,
            page: 1,
            page_size: 20,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.instances.len(), 1);
        assert_eq!(output.instances[0].status, "running");
    }

    #[tokio::test]
    async fn success_empty_list() {
        let repo = Arc::new(StubInstanceRepo::new());
        let uc = ListInstancesUseCase::new(repo);

        let input = ListInstancesInput {
            status: None,
            workflow_id: None,
            initiator_id: None,
            page: 1,
            page_size: 20,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        assert!(result.unwrap().instances.is_empty());
    }

    #[tokio::test]
    async fn error_internal() {
        let repo = Arc::new(StubInstanceRepo::with_error());
        let uc = ListInstancesUseCase::new(repo);

        let input = ListInstancesInput {
            status: None,
            workflow_id: None,
            initiator_id: None,
            page: 1,
            page_size: 20,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());
    }
}

// ============================================================
// CancelInstanceUseCase tests
// ============================================================

mod cancel_instance {
    use super::*;

    #[tokio::test]
    async fn success_cancels_running_instance() {
        let inst = running_instance();
        let repo = Arc::new(StubInstanceRepo::with_instances(vec![inst]));
        let uc = CancelInstanceUseCase::new(repo.clone());

        let input = CancelInstanceInput {
            id: "inst_001".to_string(),
            reason: Some("no longer needed".to_string()),
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let cancelled = result.unwrap();
        assert_eq!(cancelled.status, "cancelled");
        assert!(cancelled.completed_at.is_some());
        assert!(cancelled.current_step_id.is_none());

        // Verify persisted
        let instances = repo.instances.read().await;
        assert_eq!(instances[0].status, "cancelled");
    }

    #[tokio::test]
    async fn error_not_found() {
        let repo = Arc::new(StubInstanceRepo::new());
        let uc = CancelInstanceUseCase::new(repo);

        let input = CancelInstanceInput {
            id: "inst_missing".to_string(),
            reason: None,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("not found"));
    }

    #[tokio::test]
    async fn error_invalid_status_completed() {
        let inst = completed_instance();
        let repo = Arc::new(StubInstanceRepo::with_instances(vec![inst]));
        let uc = CancelInstanceUseCase::new(repo);

        let input = CancelInstanceInput {
            id: "inst_completed".to_string(),
            reason: None,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("invalid status"));
    }

    #[tokio::test]
    async fn error_invalid_status_already_cancelled() {
        let mut inst = running_instance();
        inst.cancel();
        let repo = Arc::new(StubInstanceRepo::with_instances(vec![inst]));
        let uc = CancelInstanceUseCase::new(repo);

        let input = CancelInstanceInput {
            id: "inst_001".to_string(),
            reason: None,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("invalid status"));
    }

    #[tokio::test]
    async fn error_invalid_status_failed() {
        let mut inst = running_instance();
        inst.fail();
        let repo = Arc::new(StubInstanceRepo::with_instances(vec![inst]));
        let uc = CancelInstanceUseCase::new(repo);

        let input = CancelInstanceInput {
            id: "inst_001".to_string(),
            reason: None,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("invalid status"));
    }

    #[tokio::test]
    async fn error_internal() {
        let repo = Arc::new(StubInstanceRepo::with_error());
        let uc = CancelInstanceUseCase::new(repo);

        let input = CancelInstanceInput {
            id: "inst_001".to_string(),
            reason: None,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("internal error"));
    }
}

// ============================================================
// ApproveTaskUseCase tests
// ============================================================

mod approve_task {
    use super::*;

    #[tokio::test]
    async fn success_advances_to_next_step() {
        let task = assigned_task();
        let inst = running_instance();
        let def = two_step_definition();

        let task_repo = Arc::new(StubTaskRepo::with_tasks(vec![task]));
        let inst_repo = Arc::new(StubInstanceRepo::with_instances(vec![inst]));
        let def_repo = Arc::new(StubDefinitionRepo::with_definitions(vec![def]));
        let publisher = Arc::new(StubEventPublisher);

        let uc = ApproveTaskUseCase::new(
            task_repo.clone(),
            inst_repo.clone(),
            def_repo,
            publisher,
        );

        let input = ApproveTaskInput {
            task_id: "task_001".to_string(),
            actor_id: "user-002".to_string(),
            comment: Some("Approved".to_string()),
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.task.status, "approved");
        assert_eq!(output.task.actor_id, Some("user-002".to_string()));
        assert_eq!(output.task.comment, Some("Approved".to_string()));
        assert!(output.task.decided_at.is_some());
        assert!(output.next_task.is_some());
        assert_eq!(output.instance_status, "running");

        let next_task = output.next_task.unwrap();
        assert_eq!(next_task.step_id, "step-2");
        assert_eq!(next_task.step_name, "Finance Approval");
        assert_eq!(next_task.status, "pending");

        // Verify next task was persisted
        let tasks = task_repo.tasks.read().await;
        assert_eq!(tasks.len(), 2); // original + new

        // Verify instance was updated with new step
        let instances = inst_repo.instances.read().await;
        assert_eq!(
            instances[0].current_step_id,
            Some("step-2".to_string())
        );
    }

    #[tokio::test]
    async fn success_completes_instance_on_last_step() {
        let mut task = assigned_task();
        task.step_id = "step-2".to_string();
        task.step_name = "Finance Approval".to_string();

        let mut inst = running_instance();
        inst.current_step_id = Some("step-2".to_string());

        let def = two_step_definition();

        let task_repo = Arc::new(StubTaskRepo::with_tasks(vec![task]));
        let inst_repo = Arc::new(StubInstanceRepo::with_instances(vec![inst]));
        let def_repo = Arc::new(StubDefinitionRepo::with_definitions(vec![def]));
        let publisher = Arc::new(StubEventPublisher);

        let uc = ApproveTaskUseCase::new(task_repo, inst_repo.clone(), def_repo, publisher);

        let input = ApproveTaskInput {
            task_id: "task_001".to_string(),
            actor_id: "user-003".to_string(),
            comment: None,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.task.status, "approved");
        assert!(output.next_task.is_none());
        assert_eq!(output.instance_status, "completed");

        // Verify instance was completed
        let instances = inst_repo.instances.read().await;
        assert_eq!(instances[0].status, "completed");
        assert!(instances[0].completed_at.is_some());
    }

    #[tokio::test]
    async fn success_completes_instance_with_single_step_workflow() {
        let task = WorkflowTask::new(
            "task_single".to_string(),
            "inst_single".to_string(),
            "step-1".to_string(),
            "Approval".to_string(),
            Some("user-002".to_string()),
            None,
        );

        let inst = WorkflowInstance::new(
            "inst_single".to_string(),
            "wf_002".to_string(),
            "simple-approval".to_string(),
            "Simple Request".to_string(),
            "user-001".to_string(),
            Some("step-1".to_string()),
            serde_json::json!({}),
        );

        let def = single_step_definition();

        let task_repo = Arc::new(StubTaskRepo::with_tasks(vec![task]));
        let inst_repo = Arc::new(StubInstanceRepo::with_instances(vec![inst]));
        let def_repo = Arc::new(StubDefinitionRepo::with_definitions(vec![def]));
        let publisher = Arc::new(StubEventPublisher);

        let uc = ApproveTaskUseCase::new(task_repo, inst_repo, def_repo, publisher);

        let input = ApproveTaskInput {
            task_id: "task_single".to_string(),
            actor_id: "user-002".to_string(),
            comment: Some("OK".to_string()),
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.next_task.is_none());
        assert_eq!(output.instance_status, "completed");
    }

    #[tokio::test]
    async fn error_task_not_found() {
        let task_repo = Arc::new(StubTaskRepo::new());
        let inst_repo = Arc::new(StubInstanceRepo::new());
        let def_repo = Arc::new(StubDefinitionRepo::new());
        let publisher = Arc::new(StubEventPublisher);

        let uc = ApproveTaskUseCase::new(task_repo, inst_repo, def_repo, publisher);

        let input = ApproveTaskInput {
            task_id: "task_missing".to_string(),
            actor_id: "user-002".to_string(),
            comment: None,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("not found"));
    }

    #[tokio::test]
    async fn error_invalid_status_already_approved() {
        let mut task = assigned_task();
        task.approve("prev-actor".to_string(), None);

        let task_repo = Arc::new(StubTaskRepo::with_tasks(vec![task]));
        let inst_repo = Arc::new(StubInstanceRepo::new());
        let def_repo = Arc::new(StubDefinitionRepo::new());
        let publisher = Arc::new(StubEventPublisher);

        let uc = ApproveTaskUseCase::new(task_repo, inst_repo, def_repo, publisher);

        let input = ApproveTaskInput {
            task_id: "task_001".to_string(),
            actor_id: "user-002".to_string(),
            comment: None,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("invalid task status"));
    }

    #[tokio::test]
    async fn error_invalid_status_rejected() {
        let mut task = assigned_task();
        task.reject("prev-actor".to_string(), None);

        let task_repo = Arc::new(StubTaskRepo::with_tasks(vec![task]));
        let inst_repo = Arc::new(StubInstanceRepo::new());
        let def_repo = Arc::new(StubDefinitionRepo::new());
        let publisher = Arc::new(StubEventPublisher);

        let uc = ApproveTaskUseCase::new(task_repo, inst_repo, def_repo, publisher);

        let input = ApproveTaskInput {
            task_id: "task_001".to_string(),
            actor_id: "user-002".to_string(),
            comment: None,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("invalid task status"));
    }

    #[tokio::test]
    async fn error_internal_task_repo_failure() {
        let task_repo = Arc::new(StubTaskRepo::with_error());
        let inst_repo = Arc::new(StubInstanceRepo::new());
        let def_repo = Arc::new(StubDefinitionRepo::new());
        let publisher = Arc::new(StubEventPublisher);

        let uc = ApproveTaskUseCase::new(task_repo, inst_repo, def_repo, publisher);

        let input = ApproveTaskInput {
            task_id: "task_001".to_string(),
            actor_id: "user-002".to_string(),
            comment: None,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("internal error"));
    }

    #[tokio::test]
    async fn publish_failure_does_not_fail_usecase() {
        let task = assigned_task();
        let inst = running_instance();
        let def = two_step_definition();

        let task_repo = Arc::new(StubTaskRepo::with_tasks(vec![task]));
        let inst_repo = Arc::new(StubInstanceRepo::with_instances(vec![inst]));
        let def_repo = Arc::new(StubDefinitionRepo::with_definitions(vec![def]));
        let publisher = Arc::new(FailingEventPublisher);

        let uc = ApproveTaskUseCase::new(task_repo, inst_repo, def_repo, publisher);

        let input = ApproveTaskInput {
            task_id: "task_001".to_string(),
            actor_id: "user-002".to_string(),
            comment: Some("Approved".to_string()),
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());
    }
}

// ============================================================
// RejectTaskUseCase tests
// ============================================================

mod reject_task {
    use super::*;

    #[tokio::test]
    async fn success_fails_instance_on_terminal_reject() {
        // step-1's on_reject is "end", so rejecting should fail the instance
        let task = assigned_task();
        let inst = running_instance();
        let def = two_step_definition();

        let task_repo = Arc::new(StubTaskRepo::with_tasks(vec![task]));
        let inst_repo = Arc::new(StubInstanceRepo::with_instances(vec![inst]));
        let def_repo = Arc::new(StubDefinitionRepo::with_definitions(vec![def]));
        let publisher = Arc::new(StubEventPublisher);

        let uc = RejectTaskUseCase::new(task_repo, inst_repo.clone(), def_repo, publisher);

        let input = RejectTaskInput {
            task_id: "task_001".to_string(),
            actor_id: "user-002".to_string(),
            comment: Some("Too expensive".to_string()),
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.task.status, "rejected");
        assert_eq!(output.task.comment, Some("Too expensive".to_string()));
        assert!(output.next_task.is_none());
        assert_eq!(output.instance_status, "failed");

        // Verify instance was failed
        let instances = inst_repo.instances.read().await;
        assert_eq!(instances[0].status, "failed");
        assert!(instances[0].completed_at.is_some());
    }

    #[tokio::test]
    async fn success_remands_to_previous_step() {
        // step-2's on_reject is "step-1", so rejecting should create a new task for step-1
        let mut task = assigned_task();
        task.step_id = "step-2".to_string();
        task.step_name = "Finance Approval".to_string();

        let mut inst = running_instance();
        inst.current_step_id = Some("step-2".to_string());

        let def = two_step_definition();

        let task_repo = Arc::new(StubTaskRepo::with_tasks(vec![task]));
        let inst_repo = Arc::new(StubInstanceRepo::with_instances(vec![inst]));
        let def_repo = Arc::new(StubDefinitionRepo::with_definitions(vec![def]));
        let publisher = Arc::new(StubEventPublisher);

        let uc = RejectTaskUseCase::new(
            task_repo.clone(),
            inst_repo.clone(),
            def_repo,
            publisher,
        );

        let input = RejectTaskInput {
            task_id: "task_001".to_string(),
            actor_id: "user-003".to_string(),
            comment: Some("Budget exceeded".to_string()),
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.task.status, "rejected");
        assert!(output.next_task.is_some());
        assert_eq!(output.instance_status, "running");

        let next_task = output.next_task.unwrap();
        assert_eq!(next_task.step_id, "step-1");
        assert_eq!(next_task.step_name, "Manager Approval");

        // Verify instance still running with step-1
        let instances = inst_repo.instances.read().await;
        assert_eq!(instances[0].status, "running");
        assert_eq!(
            instances[0].current_step_id,
            Some("step-1".to_string())
        );

        // Verify new task created
        let tasks = task_repo.tasks.read().await;
        assert_eq!(tasks.len(), 2);
    }

    #[tokio::test]
    async fn error_task_not_found() {
        let task_repo = Arc::new(StubTaskRepo::new());
        let inst_repo = Arc::new(StubInstanceRepo::new());
        let def_repo = Arc::new(StubDefinitionRepo::new());
        let publisher = Arc::new(StubEventPublisher);

        let uc = RejectTaskUseCase::new(task_repo, inst_repo, def_repo, publisher);

        let input = RejectTaskInput {
            task_id: "task_missing".to_string(),
            actor_id: "user-002".to_string(),
            comment: None,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("not found"));
    }

    #[tokio::test]
    async fn error_invalid_status() {
        let mut task = assigned_task();
        task.approve("prev".to_string(), None);

        let task_repo = Arc::new(StubTaskRepo::with_tasks(vec![task]));
        let inst_repo = Arc::new(StubInstanceRepo::new());
        let def_repo = Arc::new(StubDefinitionRepo::new());
        let publisher = Arc::new(StubEventPublisher);

        let uc = RejectTaskUseCase::new(task_repo, inst_repo, def_repo, publisher);

        let input = RejectTaskInput {
            task_id: "task_001".to_string(),
            actor_id: "user-002".to_string(),
            comment: None,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("invalid task status"));
    }

    #[tokio::test]
    async fn publish_failure_does_not_fail_usecase() {
        let task = assigned_task();
        let inst = running_instance();
        let def = two_step_definition();

        let task_repo = Arc::new(StubTaskRepo::with_tasks(vec![task]));
        let inst_repo = Arc::new(StubInstanceRepo::with_instances(vec![inst]));
        let def_repo = Arc::new(StubDefinitionRepo::with_definitions(vec![def]));
        let publisher = Arc::new(FailingEventPublisher);

        let uc = RejectTaskUseCase::new(task_repo, inst_repo, def_repo, publisher);

        let input = RejectTaskInput {
            task_id: "task_001".to_string(),
            actor_id: "user-002".to_string(),
            comment: Some("Rejected".to_string()),
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());
    }
}

// ============================================================
// ReassignTaskUseCase tests
// ============================================================

mod reassign_task {
    use super::*;

    #[tokio::test]
    async fn success_reassigns_assigned_task() {
        let task = assigned_task();
        let repo = Arc::new(StubTaskRepo::with_tasks(vec![task]));
        let uc = ReassignTaskUseCase::new(repo.clone());

        let input = ReassignTaskInput {
            task_id: "task_001".to_string(),
            new_assignee_id: "user-003".to_string(),
            reason: Some("on vacation".to_string()),
            actor_id: "user-002".to_string(),
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.previous_assignee_id, Some("user-002".to_string()));
        assert_eq!(output.task.assignee_id, Some("user-003".to_string()));
        assert_eq!(output.task.status, "assigned");

        // Verify persisted
        let tasks = repo.tasks.read().await;
        assert_eq!(tasks[0].assignee_id, Some("user-003".to_string()));
    }

    #[tokio::test]
    async fn success_reassigns_pending_task() {
        let task = pending_task();
        let repo = Arc::new(StubTaskRepo::with_tasks(vec![task]));
        let uc = ReassignTaskUseCase::new(repo);

        let input = ReassignTaskInput {
            task_id: "task_002".to_string(),
            new_assignee_id: "user-003".to_string(),
            reason: None,
            actor_id: "admin".to_string(),
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.previous_assignee_id, None);
        assert_eq!(output.task.assignee_id, Some("user-003".to_string()));
        assert_eq!(output.task.status, "assigned");
    }

    #[tokio::test]
    async fn error_task_not_found() {
        let repo = Arc::new(StubTaskRepo::new());
        let uc = ReassignTaskUseCase::new(repo);

        let input = ReassignTaskInput {
            task_id: "task_missing".to_string(),
            new_assignee_id: "user-003".to_string(),
            reason: None,
            actor_id: "admin".to_string(),
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("not found"));
    }

    #[tokio::test]
    async fn error_invalid_status_approved() {
        let mut task = assigned_task();
        task.approve("prev".to_string(), None);

        let repo = Arc::new(StubTaskRepo::with_tasks(vec![task]));
        let uc = ReassignTaskUseCase::new(repo);

        let input = ReassignTaskInput {
            task_id: "task_001".to_string(),
            new_assignee_id: "user-003".to_string(),
            reason: None,
            actor_id: "admin".to_string(),
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("invalid task status"));
    }

    #[tokio::test]
    async fn error_invalid_status_rejected() {
        let mut task = assigned_task();
        task.reject("prev".to_string(), None);

        let repo = Arc::new(StubTaskRepo::with_tasks(vec![task]));
        let uc = ReassignTaskUseCase::new(repo);

        let input = ReassignTaskInput {
            task_id: "task_001".to_string(),
            new_assignee_id: "user-003".to_string(),
            reason: None,
            actor_id: "admin".to_string(),
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("invalid task status"));
    }

    #[tokio::test]
    async fn error_internal() {
        let repo = Arc::new(StubTaskRepo::with_error());
        let uc = ReassignTaskUseCase::new(repo);

        let input = ReassignTaskInput {
            task_id: "task_001".to_string(),
            new_assignee_id: "user-003".to_string(),
            reason: None,
            actor_id: "admin".to_string(),
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("internal error"));
    }
}

// ============================================================
// ListTasksUseCase tests
// ============================================================

mod list_tasks {
    use super::*;

    #[tokio::test]
    async fn success_returns_all() {
        let tasks = vec![assigned_task(), pending_task()];
        let repo = Arc::new(StubTaskRepo::with_tasks(tasks));
        let uc = ListTasksUseCase::new(repo);

        let input = ListTasksInput {
            assignee_id: None,
            status: None,
            instance_id: None,
            overdue_only: false,
            page: 1,
            page_size: 20,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.tasks.len(), 2);
        assert_eq!(output.total_count, 2);
    }

    #[tokio::test]
    async fn success_filters_by_assignee() {
        let tasks = vec![assigned_task(), pending_task()];
        let repo = Arc::new(StubTaskRepo::with_tasks(tasks));
        let uc = ListTasksUseCase::new(repo);

        let input = ListTasksInput {
            assignee_id: Some("user-002".to_string()),
            status: None,
            instance_id: None,
            overdue_only: false,
            page: 1,
            page_size: 20,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.tasks.len(), 1);
        assert_eq!(output.tasks[0].assignee_id, Some("user-002".to_string()));
    }

    #[tokio::test]
    async fn success_empty_list() {
        let repo = Arc::new(StubTaskRepo::new());
        let uc = ListTasksUseCase::new(repo);

        let input = ListTasksInput {
            assignee_id: None,
            status: None,
            instance_id: None,
            overdue_only: false,
            page: 1,
            page_size: 20,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        assert!(result.unwrap().tasks.is_empty());
    }

    #[tokio::test]
    async fn error_internal() {
        let repo = Arc::new(StubTaskRepo::with_error());
        let uc = ListTasksUseCase::new(repo);

        let input = ListTasksInput {
            assignee_id: None,
            status: None,
            instance_id: None,
            overdue_only: false,
            page: 1,
            page_size: 20,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());
    }
}

// ============================================================
// CheckOverdueTasksUseCase tests
// ============================================================

mod check_overdue_tasks {
    use super::*;

    #[tokio::test]
    async fn success_no_overdue_tasks() {
        let repo = Arc::new(StubTaskRepo::new());
        let publisher = Arc::new(StubNotificationPublisher);

        let uc = CheckOverdueTasksUseCase::new(repo, publisher);
        let result = uc.execute().await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.overdue_tasks.is_empty());
        assert_eq!(output.count, 0);
        assert_eq!(output.published_count, 0);
    }

    #[tokio::test]
    async fn success_with_overdue_tasks() {
        let task = overdue_task();
        let repo = Arc::new(StubTaskRepo::with_tasks(vec![task]));
        let publisher = Arc::new(StubNotificationPublisher);

        let uc = CheckOverdueTasksUseCase::new(repo, publisher);
        let result = uc.execute().await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.count, 1);
        assert_eq!(output.published_count, 1);
    }

    #[tokio::test]
    async fn publish_failure_is_counted() {
        let task = overdue_task();
        let repo = Arc::new(StubTaskRepo::with_tasks(vec![task]));
        let publisher = Arc::new(FailingNotificationPublisher);

        let uc = CheckOverdueTasksUseCase::new(repo, publisher);
        let result = uc.execute().await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.count, 1);
        assert_eq!(output.published_count, 0); // failed to publish
    }

    #[tokio::test]
    async fn error_internal() {
        let repo = Arc::new(StubTaskRepo::with_error());
        let publisher = Arc::new(StubNotificationPublisher);

        let uc = CheckOverdueTasksUseCase::new(repo, publisher);
        let result = uc.execute().await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("internal error"));
    }
}

// ============================================================
// End-to-end workflow state transition tests
// ============================================================

mod workflow_state_transitions {
    use super::*;

    /// Tests the full lifecycle: create workflow -> start instance -> approve -> complete
    #[tokio::test]
    async fn full_single_step_lifecycle() {
        // 1. Create workflow definition
        let def_repo = Arc::new(StubDefinitionRepo::new());
        let create_uc = CreateWorkflowUseCase::new(def_repo.clone());

        let create_input = CreateWorkflowInput {
            name: "leave-request".to_string(),
            description: "Leave request approval".to_string(),
            enabled: true,
            steps: sample_steps(),
        };

        let def = create_uc.execute(&create_input).await.unwrap();
        assert_eq!(def.version, 1);

        // 2. Start instance
        let inst_repo = Arc::new(StubInstanceRepo::new());
        let task_repo = Arc::new(StubTaskRepo::new());
        let publisher = Arc::new(StubEventPublisher);

        let start_uc = StartInstanceUseCase::new(
            def_repo.clone(),
            inst_repo.clone(),
            task_repo.clone(),
            publisher.clone(),
        );

        let start_input = StartInstanceInput {
            workflow_id: def.id.clone(),
            title: "Summer Vacation".to_string(),
            initiator_id: "user-001".to_string(),
            context: serde_json::json!({"days": 5}),
        };

        let start_output = start_uc.execute(&start_input).await.unwrap();
        assert_eq!(start_output.instance.status, "running");
        let first_task = start_output.first_task.unwrap();

        // 3. Approve the task
        let approve_uc = ApproveTaskUseCase::new(
            task_repo.clone(),
            inst_repo.clone(),
            def_repo.clone(),
            publisher.clone(),
        );

        let approve_input = ApproveTaskInput {
            task_id: first_task.id.clone(),
            actor_id: "manager-001".to_string(),
            comment: Some("Enjoy your vacation!".to_string()),
        };

        let approve_output = approve_uc.execute(&approve_input).await.unwrap();
        assert_eq!(approve_output.task.status, "approved");
        assert!(approve_output.next_task.is_none()); // single step, so done
        assert_eq!(approve_output.instance_status, "completed");

        // 4. Verify final state
        let instances = inst_repo.instances.read().await;
        assert_eq!(instances[0].status, "completed");
        assert!(instances[0].completed_at.is_some());
    }

    /// Tests: create workflow -> start instance -> approve step-1 -> approve step-2 -> complete
    #[tokio::test]
    async fn full_two_step_lifecycle_approve_approve() {
        let def_repo = Arc::new(StubDefinitionRepo::with_definitions(vec![
            two_step_definition(),
        ]));
        let inst_repo = Arc::new(StubInstanceRepo::new());
        let task_repo = Arc::new(StubTaskRepo::new());
        let publisher = Arc::new(StubEventPublisher);

        // Start
        let start_uc = StartInstanceUseCase::new(
            def_repo.clone(),
            inst_repo.clone(),
            task_repo.clone(),
            publisher.clone(),
        );

        let start_output = start_uc
            .execute(&StartInstanceInput {
                workflow_id: "wf_001".to_string(),
                title: "PC Purchase".to_string(),
                initiator_id: "user-001".to_string(),
                context: serde_json::json!({"item": "laptop"}),
            })
            .await
            .unwrap();

        let task1 = start_output.first_task.unwrap();
        assert_eq!(task1.step_id, "step-1");

        // Approve step-1
        let approve_uc = ApproveTaskUseCase::new(
            task_repo.clone(),
            inst_repo.clone(),
            def_repo.clone(),
            publisher.clone(),
        );

        let approve1_output = approve_uc
            .execute(&ApproveTaskInput {
                task_id: task1.id.clone(),
                actor_id: "manager-001".to_string(),
                comment: None,
            })
            .await
            .unwrap();

        assert_eq!(approve1_output.instance_status, "running");
        let task2 = approve1_output.next_task.unwrap();
        assert_eq!(task2.step_id, "step-2");

        // Approve step-2
        let approve2_output = approve_uc
            .execute(&ApproveTaskInput {
                task_id: task2.id.clone(),
                actor_id: "finance-001".to_string(),
                comment: Some("Budget approved".to_string()),
            })
            .await
            .unwrap();

        assert_eq!(approve2_output.instance_status, "completed");
        assert!(approve2_output.next_task.is_none());
    }

    /// Tests: start instance -> reject step-1 -> instance fails
    #[tokio::test]
    async fn two_step_lifecycle_reject_at_first_step() {
        let def_repo = Arc::new(StubDefinitionRepo::with_definitions(vec![
            two_step_definition(),
        ]));
        let inst_repo = Arc::new(StubInstanceRepo::new());
        let task_repo = Arc::new(StubTaskRepo::new());
        let publisher = Arc::new(StubEventPublisher);

        // Start
        let start_uc = StartInstanceUseCase::new(
            def_repo.clone(),
            inst_repo.clone(),
            task_repo.clone(),
            publisher.clone(),
        );

        let start_output = start_uc
            .execute(&StartInstanceInput {
                workflow_id: "wf_001".to_string(),
                title: "PC Purchase".to_string(),
                initiator_id: "user-001".to_string(),
                context: serde_json::json!({}),
            })
            .await
            .unwrap();

        let task1 = start_output.first_task.unwrap();

        // Reject step-1 (on_reject = "end")
        let reject_uc = RejectTaskUseCase::new(
            task_repo.clone(),
            inst_repo.clone(),
            def_repo.clone(),
            publisher.clone(),
        );

        let reject_output = reject_uc
            .execute(&RejectTaskInput {
                task_id: task1.id.clone(),
                actor_id: "manager-001".to_string(),
                comment: Some("Not justified".to_string()),
            })
            .await
            .unwrap();

        assert_eq!(reject_output.instance_status, "failed");
        assert!(reject_output.next_task.is_none());
    }

    /// Tests: start -> approve step-1 -> reject step-2 -> remand to step-1
    #[tokio::test]
    async fn two_step_lifecycle_approve_then_reject_remands() {
        let def_repo = Arc::new(StubDefinitionRepo::with_definitions(vec![
            two_step_definition(),
        ]));
        let inst_repo = Arc::new(StubInstanceRepo::new());
        let task_repo = Arc::new(StubTaskRepo::new());
        let publisher = Arc::new(StubEventPublisher);

        // Start
        let start_uc = StartInstanceUseCase::new(
            def_repo.clone(),
            inst_repo.clone(),
            task_repo.clone(),
            publisher.clone(),
        );

        let start_output = start_uc
            .execute(&StartInstanceInput {
                workflow_id: "wf_001".to_string(),
                title: "Equipment".to_string(),
                initiator_id: "user-001".to_string(),
                context: serde_json::json!({}),
            })
            .await
            .unwrap();

        let task1 = start_output.first_task.unwrap();

        // Approve step-1
        let approve_uc = ApproveTaskUseCase::new(
            task_repo.clone(),
            inst_repo.clone(),
            def_repo.clone(),
            publisher.clone(),
        );

        let approve_output = approve_uc
            .execute(&ApproveTaskInput {
                task_id: task1.id.clone(),
                actor_id: "manager-001".to_string(),
                comment: None,
            })
            .await
            .unwrap();

        let task2 = approve_output.next_task.unwrap();
        assert_eq!(task2.step_id, "step-2");

        // Reject step-2 (on_reject = "step-1", remands back)
        let reject_uc = RejectTaskUseCase::new(
            task_repo.clone(),
            inst_repo.clone(),
            def_repo.clone(),
            publisher.clone(),
        );

        let reject_output = reject_uc
            .execute(&RejectTaskInput {
                task_id: task2.id.clone(),
                actor_id: "finance-001".to_string(),
                comment: Some("Budget issue".to_string()),
            })
            .await
            .unwrap();

        assert_eq!(reject_output.instance_status, "running");
        let remanded_task = reject_output.next_task.unwrap();
        assert_eq!(remanded_task.step_id, "step-1");
        assert_eq!(remanded_task.step_name, "Manager Approval");

        // Verify instance is back at step-1
        let instances = inst_repo.instances.read().await;
        assert_eq!(instances[0].current_step_id, Some("step-1".to_string()));
    }

    /// Tests: start -> cancel
    #[tokio::test]
    async fn start_then_cancel() {
        let def_repo = Arc::new(StubDefinitionRepo::with_definitions(vec![
            single_step_definition(),
        ]));
        let inst_repo = Arc::new(StubInstanceRepo::new());
        let task_repo = Arc::new(StubTaskRepo::new());
        let publisher = Arc::new(StubEventPublisher);

        let start_uc = StartInstanceUseCase::new(
            def_repo.clone(),
            inst_repo.clone(),
            task_repo.clone(),
            publisher,
        );

        let start_output = start_uc
            .execute(&StartInstanceInput {
                workflow_id: "wf_002".to_string(),
                title: "Test".to_string(),
                initiator_id: "user-001".to_string(),
                context: serde_json::json!({}),
            })
            .await
            .unwrap();

        let instance_id = start_output.instance.id.clone();

        // Cancel
        let cancel_uc = CancelInstanceUseCase::new(inst_repo.clone());
        let cancel_result = cancel_uc
            .execute(&CancelInstanceInput {
                id: instance_id.clone(),
                reason: Some("changed my mind".to_string()),
            })
            .await;

        assert!(cancel_result.is_ok());
        let cancelled = cancel_result.unwrap();
        assert_eq!(cancelled.status, "cancelled");

        // Cannot cancel again
        let cancel_again = cancel_uc
            .execute(&CancelInstanceInput {
                id: instance_id,
                reason: None,
            })
            .await;

        assert!(cancel_again.is_err());
    }
}
