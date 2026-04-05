// インメモリリポジトリ実装
// テスト・開発用のインメモリストア。RLS は不要なため tenant_id パラメータは受け取るが無視する
// RUST-CRIT-001 対応: トレイトシグネチャに合わせて tenant_id パラメータを追加する

use std::collections::HashMap;

use crate::domain::entity::workflow_definition::WorkflowDefinition;
use crate::domain::entity::workflow_instance::WorkflowInstance;
use crate::domain::entity::workflow_task::WorkflowTask;
use crate::domain::repository::{
    WorkflowDefinitionRepository, WorkflowInstanceRepository, WorkflowTaskRepository,
};

pub struct InMemoryWorkflowDefinitionRepository {
    definitions: tokio::sync::RwLock<HashMap<String, WorkflowDefinition>>,
}

impl Default for InMemoryWorkflowDefinitionRepository {
    fn default() -> Self {
        Self {
            definitions: tokio::sync::RwLock::new(HashMap::new()),
        }
    }
}

impl InMemoryWorkflowDefinitionRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait::async_trait]
impl WorkflowDefinitionRepository for InMemoryWorkflowDefinitionRepository {
    // tenant_id はインメモリ実装では使用しないが、トレイトシグネチャに合わせて受け取る
    async fn find_by_id(&self, _tenant_id: &str, id: &str) -> anyhow::Result<Option<WorkflowDefinition>> {
        let defs = self.definitions.read().await;
        Ok(defs.get(id).cloned())
    }

    async fn find_by_name(&self, _tenant_id: &str, name: &str) -> anyhow::Result<Option<WorkflowDefinition>> {
        let defs = self.definitions.read().await;
        Ok(defs.values().find(|d| d.name == name).cloned())
    }

    async fn find_all(
        &self,
        _tenant_id: &str,
        enabled_only: bool,
        _page: u32,
        _page_size: u32,
    ) -> anyhow::Result<(Vec<WorkflowDefinition>, u64)> {
        let defs = self.definitions.read().await;
        let results: Vec<_> = if enabled_only {
            defs.values().filter(|d| d.enabled).cloned().collect()
        } else {
            defs.values().cloned().collect()
        };
        let total = results.len() as u64;
        Ok((results, total))
    }

    async fn create(&self, _tenant_id: &str, definition: &WorkflowDefinition) -> anyhow::Result<()> {
        let mut defs = self.definitions.write().await;
        defs.insert(definition.id.clone(), definition.clone());
        Ok(())
    }

    async fn update(&self, _tenant_id: &str, definition: &WorkflowDefinition) -> anyhow::Result<()> {
        let mut defs = self.definitions.write().await;
        defs.insert(definition.id.clone(), definition.clone());
        Ok(())
    }

    async fn delete(&self, _tenant_id: &str, id: &str) -> anyhow::Result<bool> {
        let mut defs = self.definitions.write().await;
        Ok(defs.remove(id).is_some())
    }
}

pub struct InMemoryWorkflowInstanceRepository {
    instances: tokio::sync::RwLock<HashMap<String, WorkflowInstance>>,
}

impl Default for InMemoryWorkflowInstanceRepository {
    fn default() -> Self {
        Self {
            instances: tokio::sync::RwLock::new(HashMap::new()),
        }
    }
}

impl InMemoryWorkflowInstanceRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait::async_trait]
impl WorkflowInstanceRepository for InMemoryWorkflowInstanceRepository {
    // tenant_id はインメモリ実装では使用しないが、トレイトシグネチャに合わせて受け取る
    async fn find_by_id(&self, _tenant_id: &str, id: &str) -> anyhow::Result<Option<WorkflowInstance>> {
        let instances = self.instances.read().await;
        Ok(instances.get(id).cloned())
    }

    async fn find_all(
        &self,
        _tenant_id: &str,
        status: Option<String>,
        workflow_id: Option<String>,
        initiator_id: Option<String>,
        _page: u32,
        _page_size: u32,
    ) -> anyhow::Result<(Vec<WorkflowInstance>, u64)> {
        let instances = self.instances.read().await;
        let results: Vec<_> = instances
            .values()
            .filter(|i| {
                status.as_deref().is_none_or(|s| i.status == s)
                    && workflow_id.as_deref().is_none_or(|w| i.workflow_id == w)
                    && initiator_id
                        .as_deref()
                        .is_none_or(|init| i.initiator_id == init)
            })
            .cloned()
            .collect();
        let total = results.len() as u64;
        Ok((results, total))
    }

    async fn create(&self, _tenant_id: &str, instance: &WorkflowInstance) -> anyhow::Result<()> {
        let mut instances = self.instances.write().await;
        instances.insert(instance.id.clone(), instance.clone());
        Ok(())
    }

    async fn update(&self, _tenant_id: &str, instance: &WorkflowInstance) -> anyhow::Result<()> {
        let mut instances = self.instances.write().await;
        instances.insert(instance.id.clone(), instance.clone());
        Ok(())
    }
}

pub struct InMemoryWorkflowTaskRepository {
    tasks: tokio::sync::RwLock<HashMap<String, WorkflowTask>>,
}

impl Default for InMemoryWorkflowTaskRepository {
    fn default() -> Self {
        Self {
            tasks: tokio::sync::RwLock::new(HashMap::new()),
        }
    }
}

impl InMemoryWorkflowTaskRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait::async_trait]
impl WorkflowTaskRepository for InMemoryWorkflowTaskRepository {
    // tenant_id はインメモリ実装では使用しないが、トレイトシグネチャに合わせて受け取る
    async fn find_by_id(&self, _tenant_id: &str, id: &str) -> anyhow::Result<Option<WorkflowTask>> {
        let tasks = self.tasks.read().await;
        Ok(tasks.get(id).cloned())
    }

    async fn find_all(
        &self,
        _tenant_id: &str,
        assignee_id: Option<String>,
        status: Option<String>,
        instance_id: Option<String>,
        overdue_only: bool,
        _page: u32,
        _page_size: u32,
    ) -> anyhow::Result<(Vec<WorkflowTask>, u64)> {
        let tasks = self.tasks.read().await;
        let results: Vec<_> = tasks
            .values()
            .filter(|t| {
                assignee_id
                    .as_deref()
                    .is_none_or(|a| t.assignee_id.as_deref() == Some(a))
                    && status.as_deref().is_none_or(|s| t.status == s)
                    && instance_id.as_deref().is_none_or(|i| t.instance_id == i)
                    && (!overdue_only || t.is_overdue())
            })
            .cloned()
            .collect();
        let total = results.len() as u64;
        Ok((results, total))
    }

    async fn find_overdue(&self) -> anyhow::Result<Vec<WorkflowTask>> {
        let tasks = self.tasks.read().await;
        Ok(tasks.values().filter(|t| t.is_overdue()).cloned().collect())
    }

    async fn create(&self, _tenant_id: &str, task: &WorkflowTask) -> anyhow::Result<()> {
        let mut tasks = self.tasks.write().await;
        tasks.insert(task.id.clone(), task.clone());
        Ok(())
    }

    async fn update(&self, _tenant_id: &str, task: &WorkflowTask) -> anyhow::Result<()> {
        let mut tasks = self.tasks.write().await;
        tasks.insert(task.id.clone(), task.clone());
        Ok(())
    }
}
