use std::sync::Arc;

use crate::domain::entity::workflow::WorkflowDefinition;
use crate::domain::repository::WorkflowRepository;

/// RegisterWorkflowUseCase はワークフロー登録を担う。
pub struct RegisterWorkflowUseCase {
    workflow_repo: Arc<dyn WorkflowRepository>,
}

impl RegisterWorkflowUseCase {
    pub fn new(workflow_repo: Arc<dyn WorkflowRepository>) -> Self {
        Self { workflow_repo }
    }

    /// YAML文字列からワークフローを登録する。名前とステップ数を返す。
    pub async fn execute(&self, yaml_content: String) -> anyhow::Result<(String, usize)> {
        let def = WorkflowDefinition::from_yaml(&yaml_content)?;
        let name = def.name.clone();
        let step_count = def.steps.len();
        self.workflow_repo.register(def).await?;

        tracing::info!(name = %name, steps = step_count, "workflow registered");
        Ok((name, step_count))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::workflow_repository::MockWorkflowRepository;

    #[tokio::test]
    async fn test_register_workflow_success() {
        let mut mock = MockWorkflowRepository::new();
        mock.expect_register().returning(|_| Ok(()));

        let uc = RegisterWorkflowUseCase::new(Arc::new(mock));
        let yaml = r#"
name: test-workflow
steps:
  - name: step1
    service: svc
    method: Svc.Do
"#;
        let (name, count) = uc.execute(yaml.to_string()).await.unwrap();
        assert_eq!(name, "test-workflow");
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_register_workflow_invalid_yaml() {
        let mock = MockWorkflowRepository::new();
        let uc = RegisterWorkflowUseCase::new(Arc::new(mock));
        let result = uc.execute("invalid yaml {{".to_string()).await;
        assert!(result.is_err());
    }
}
