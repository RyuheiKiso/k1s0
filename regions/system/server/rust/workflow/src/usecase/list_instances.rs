use std::sync::Arc;

use crate::domain::entity::workflow_instance::WorkflowInstance;
use crate::domain::repository::WorkflowInstanceRepository;

// RUST-CRIT-001 対応: テナント分離のため tenant_id フィールドを追加する
#[derive(Debug, Clone)]
pub struct ListInstancesInput {
    pub tenant_id: String,
    pub status: Option<String>,
    pub workflow_id: Option<String>,
    pub initiator_id: Option<String>,
    pub page: u32,
    pub page_size: u32,
}

#[derive(Debug, Clone)]
pub struct ListInstancesOutput {
    pub instances: Vec<WorkflowInstance>,
    pub total_count: u64,
    pub page: u32,
    pub page_size: u32,
    pub has_next: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum ListInstancesError {
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct ListInstancesUseCase {
    repo: Arc<dyn WorkflowInstanceRepository>,
}

impl ListInstancesUseCase {
    pub fn new(repo: Arc<dyn WorkflowInstanceRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        input: &ListInstancesInput,
    ) -> Result<ListInstancesOutput, ListInstancesError> {
        // RUST-MED-002 対応: page に上限を設けることで OFFSET の過大計算を防ぐ
        // page_size は 1〜200 にクランプして異常値を防ぐ（H-07 監査対応）
        let page = input.page.clamp(1, 10_000);
        let page_size = input.page_size.clamp(1, 200);

        // テナント分離: tenant_id を渡してRLSによるフィルタリングを有効化する
        let (instances, total_count) = self
            .repo
            .find_all(
                &input.tenant_id,
                input.status.clone(),
                input.workflow_id.clone(),
                input.initiator_id.clone(),
                page,
                page_size,
            )
            .await
            .map_err(|e| ListInstancesError::Internal(e.to_string()))?;

        let has_next = (u64::from(page) * u64::from(page_size)) < total_count;

        Ok(ListInstancesOutput {
            instances,
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
    use crate::domain::repository::workflow_instance_repository::MockWorkflowInstanceRepository;

    #[tokio::test]
    async fn success() {
        let mut mock = MockWorkflowInstanceRepository::new();
        mock.expect_find_all()
            .returning(|_, _, _, _, _, _| Ok((vec![], 0)));

        let uc = ListInstancesUseCase::new(Arc::new(mock));
        let input = ListInstancesInput {
            tenant_id: "test-tenant".to_string(),
            status: None,
            workflow_id: None,
            initiator_id: None,
            page: 1,
            page_size: 20,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.instances.is_empty());
        assert_eq!(output.total_count, 0);
    }

    #[tokio::test]
    async fn has_next_page() {
        let mut mock = MockWorkflowInstanceRepository::new();
        mock.expect_find_all()
            .returning(|_, _, _, _, _| Ok((vec![], 50)));

        let uc = ListInstancesUseCase::new(Arc::new(mock));
        let input = ListInstancesInput {
            tenant_id: "test-tenant".to_string(),
            status: Some("running".to_string()),
            workflow_id: None,
            initiator_id: None,
            page: 1,
            page_size: 20,
        };
        let result = uc.execute(&input).await;
        assert!(result.unwrap().has_next);
    }

    #[tokio::test]
    async fn internal_error() {
        let mut mock = MockWorkflowInstanceRepository::new();
        mock.expect_find_all()
            .returning(|_, _, _, _, _| Err(anyhow::anyhow!("db error")));

        let uc = ListInstancesUseCase::new(Arc::new(mock));
        let input = ListInstancesInput {
            tenant_id: "test-tenant".to_string(),
            status: None,
            workflow_id: None,
            initiator_id: None,
            page: 1,
            page_size: 20,
        };
        let result = uc.execute(&input).await;
        assert!(matches!(
            result.unwrap_err(),
            ListInstancesError::Internal(_)
        ));
    }
}
