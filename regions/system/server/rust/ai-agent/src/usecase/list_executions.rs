// 実行履歴一覧ユースケース
// エージェントIDに基づいて実行履歴を取得する

use std::sync::Arc;

use crate::domain::entity::Execution;
use crate::domain::repository::ExecutionRepository;

/// ListExecutionsUseCase は実行履歴一覧の取得を担当する
pub struct ListExecutionsUseCase {
    /// 実行リポジトリ
    execution_repo: Arc<dyn ExecutionRepository>,
}

/// 実行履歴一覧リクエスト
pub struct ListExecutionsRequest {
    pub agent_id: String,
}

/// 実行履歴一覧レスポンス
pub struct ListExecutionsResponse {
    pub executions: Vec<Execution>,
}

impl ListExecutionsUseCase {
    /// 新しいListExecutionsUseCaseを生成する
    pub fn new(execution_repo: Arc<dyn ExecutionRepository>) -> Self {
        Self { execution_repo }
    }

    /// エージェントIDで実行履歴を取得する
    pub async fn execute(
        &self,
        req: ListExecutionsRequest,
    ) -> anyhow::Result<ListExecutionsResponse> {
        let executions = self.execution_repo.find_by_agent(&req.agent_id).await?;
        Ok(ListExecutionsResponse { executions })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::{Execution, ExecutionStatus};
    use crate::domain::repository::execution_repository::MockExecutionRepository;
    use chrono::Utc;
    use uuid::Uuid;

    // テスト用の実行エンティティを生成するヘルパー関数
    fn sample_execution(agent_id: &str) -> Execution {
        let now = Utc::now();
        Execution {
            id: Uuid::new_v4().to_string(),
            agent_id: agent_id.to_string(),
            session_id: "session-001".to_string(),
            tenant_id: "tenant-001".to_string(),
            input: "テスト入力".to_string(),
            output: Some("テスト出力".to_string()),
            status: ExecutionStatus::Completed,
            steps: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    // 正常系: エージェントIDに紐づく実行履歴が返される
    #[tokio::test]
    async fn test_list_executions_success() {
        let agent_id = "agent-001";
        let executions = vec![sample_execution(agent_id), sample_execution(agent_id)];
        let executions_clone = executions.clone();

        let mut mock_repo = MockExecutionRepository::new();
        mock_repo
            .expect_find_by_agent()
            .withf(|id| id == "agent-001")
            .times(1)
            .returning(move |_| Ok(executions_clone.clone()));

        let uc = ListExecutionsUseCase::new(Arc::new(mock_repo));
        let result = uc
            .execute(ListExecutionsRequest {
                agent_id: agent_id.to_string(),
            })
            .await;

        assert!(result.is_ok());
        let resp = result.unwrap();
        assert_eq!(resp.executions.len(), 2);
        assert!(resp.executions.iter().all(|e| e.agent_id == agent_id));
    }

    // 境界値: 実行履歴が存在しない場合に空リストが返される
    #[tokio::test]
    async fn test_list_executions_empty() {
        let mut mock_repo = MockExecutionRepository::new();
        mock_repo
            .expect_find_by_agent()
            .times(1)
            .returning(|_| Ok(Vec::new()));

        let uc = ListExecutionsUseCase::new(Arc::new(mock_repo));
        let result = uc
            .execute(ListExecutionsRequest {
                agent_id: "no-such-agent".to_string(),
            })
            .await;

        assert!(result.is_ok());
        assert!(result.unwrap().executions.is_empty());
    }
}
