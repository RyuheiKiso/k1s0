use std::sync::Arc;

use crate::domain::entity::workflow_definition::WorkflowDefinition;
use crate::domain::repository::WorkflowDefinitionRepository;

// RUST-CRIT-001 対応: テナント分離のため tenant_id フィールドを追加する
#[derive(Debug, Clone)]
pub struct ListWorkflowsInput {
    pub tenant_id: String,
    pub enabled_only: bool,
    pub page: u32,
    pub page_size: u32,
}

#[derive(Debug, Clone)]
pub struct ListWorkflowsOutput {
    pub workflows: Vec<WorkflowDefinition>,
    pub total_count: u64,
    pub page: u32,
    pub page_size: u32,
    pub has_next: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum ListWorkflowsError {
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct ListWorkflowsUseCase {
    repo: Arc<dyn WorkflowDefinitionRepository>,
}

impl ListWorkflowsUseCase {
    pub fn new(repo: Arc<dyn WorkflowDefinitionRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        input: &ListWorkflowsInput,
    ) -> Result<ListWorkflowsOutput, ListWorkflowsError> {
        // RUST-MED-002 対応: page に上限を設けることで OFFSET の過大計算を防ぐ
        // page_size は 1〜200 にクランプして異常値を防ぐ（H-07 監査対応）
        let page = input.page.clamp(1, 10_000);
        let page_size = input.page_size.clamp(1, 200);

        // テナント分離: tenant_id を渡してRLSによるフィルタリングを有効化する
        let (workflows, total_count) = self
            .repo
            .find_all(&input.tenant_id, input.enabled_only, page, page_size)
            .await
            .map_err(|e| ListWorkflowsError::Internal(e.to_string()))?;

        let has_next = (page as u64 * page_size as u64) < total_count;

        Ok(ListWorkflowsOutput {
            workflows,
            total_count,
            page,
            page_size,
            has_next,
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::repository::workflow_definition_repository::MockWorkflowDefinitionRepository;

    #[tokio::test]
    async fn success() {
        let mut mock = MockWorkflowDefinitionRepository::new();
        mock.expect_find_all().returning(|_, _, _, _| {
            Ok((
                vec![WorkflowDefinition::new(
                    "wf_001".to_string(),
                    "test".to_string(),
                    "".to_string(),
                    true,
                    vec![],
                )],
                1,
            ))
        });

        let uc = ListWorkflowsUseCase::new(Arc::new(mock));
        let input = ListWorkflowsInput {
            tenant_id: "test-tenant".to_string(),
            enabled_only: false,
            page: 1,
            page_size: 20,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.workflows.len(), 1);
        assert_eq!(output.total_count, 1);
        assert!(!output.has_next);
    }

    #[tokio::test]
    async fn has_next_page() {
        let mut mock = MockWorkflowDefinitionRepository::new();
        mock.expect_find_all().returning(|_, _, _, _| Ok((vec![], 50)));

        let uc = ListWorkflowsUseCase::new(Arc::new(mock));
        let input = ListWorkflowsInput {
            tenant_id: "test-tenant".to_string(),
            enabled_only: false,
            page: 1,
            page_size: 20,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        assert!(result.unwrap().has_next);
    }

    #[tokio::test]
    async fn internal_error() {
        let mut mock = MockWorkflowDefinitionRepository::new();
        mock.expect_find_all()
            .returning(|_, _, _, _| Err(anyhow::anyhow!("db error")));

        let uc = ListWorkflowsUseCase::new(Arc::new(mock));
        let input = ListWorkflowsInput {
            tenant_id: "test-tenant".to_string(),
            enabled_only: false,
            page: 1,
            page_size: 20,
        };
        let result = uc.execute(&input).await;
        assert!(matches!(
            result.unwrap_err(),
            ListWorkflowsError::Internal(_)
        ));
    }
}
