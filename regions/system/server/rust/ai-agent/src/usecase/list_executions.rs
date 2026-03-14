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
