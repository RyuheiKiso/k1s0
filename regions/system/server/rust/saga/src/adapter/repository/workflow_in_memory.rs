use std::collections::HashMap;

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::domain::entity::workflow::WorkflowDefinition;
use crate::domain::repository::WorkflowRepository;

/// InMemoryWorkflowRepository はインメモリのワークフローリポジトリ。
pub struct InMemoryWorkflowRepository {
    workflows: RwLock<HashMap<String, WorkflowDefinition>>,
}

impl InMemoryWorkflowRepository {
    /// 新しいInMemoryWorkflowRepositoryを作成する。
    pub fn new() -> Self {
        Self {
            workflows: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryWorkflowRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl WorkflowRepository for InMemoryWorkflowRepository {
    async fn register(&self, workflow: WorkflowDefinition) -> anyhow::Result<()> {
        let mut workflows = self.workflows.write().await;
        workflows.insert(workflow.name.clone(), workflow);
        Ok(())
    }

    async fn get(&self, name: &str) -> anyhow::Result<Option<WorkflowDefinition>> {
        let workflows = self.workflows.read().await;
        Ok(workflows.get(name).cloned())
    }

    async fn list(&self) -> anyhow::Result<Vec<WorkflowDefinition>> {
        let workflows = self.workflows.read().await;
        Ok(workflows.values().cloned().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_register_and_get() {
        let repo = InMemoryWorkflowRepository::new();
        let yaml = r#"
name: test-workflow
steps:
  - name: step1
    service: svc
    method: Svc.Do
"#;
        let def = WorkflowDefinition::from_yaml(yaml).unwrap();
        repo.register(def).await.unwrap();

        let result = repo.get("test-workflow").await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "test-workflow");
    }

    #[tokio::test]
    async fn test_get_not_found() {
        let repo = InMemoryWorkflowRepository::new();
        let result = repo.get("nonexistent").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_list() {
        let repo = InMemoryWorkflowRepository::new();
        let yaml1 = r#"
name: workflow-1
steps:
  - name: step1
    service: svc
    method: Svc.Do
"#;
        let yaml2 = r#"
name: workflow-2
steps:
  - name: step1
    service: svc
    method: Svc.Do
"#;
        repo.register(WorkflowDefinition::from_yaml(yaml1).unwrap())
            .await
            .unwrap();
        repo.register(WorkflowDefinition::from_yaml(yaml2).unwrap())
            .await
            .unwrap();

        let list = repo.list().await.unwrap();
        assert_eq!(list.len(), 2);
    }
}
