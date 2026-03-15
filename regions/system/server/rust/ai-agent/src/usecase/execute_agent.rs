// エージェント実行ユースケース
// エージェント定義を取得し、ReActエンジンで実行し、結果を保存する

use std::sync::Arc;

use chrono::Utc;
use tracing::info;
use uuid::Uuid;

use crate::domain::entity::{Execution, ExecutionStatus};
use crate::domain::repository::{AgentRepository, ExecutionRepository};
use crate::domain::service::ReActEngine;
use k1s0_bb_ai_client::traits::AiClient;

/// ExecuteAgentUseCase はエージェント実行を担当する
pub struct ExecuteAgentUseCase {
    /// エージェントリポジトリ
    agent_repo: Arc<dyn AgentRepository>,
    /// 実行リポジトリ
    execution_repo: Arc<dyn ExecutionRepository>,
    /// ReActエンジン
    react_engine: Arc<ReActEngine>,
    /// AIクライアント
    ai_client: Arc<dyn AiClient>,
}

/// エージェント実行リクエスト
pub struct ExecuteAgentRequest {
    pub agent_id: String,
    pub input: String,
    pub session_id: String,
    pub tenant_id: String,
}

/// エージェント実行レスポンス
pub struct ExecuteAgentResponse {
    pub execution: Execution,
}

impl ExecuteAgentUseCase {
    /// 新しいExecuteAgentUseCaseを生成する
    pub fn new(
        agent_repo: Arc<dyn AgentRepository>,
        execution_repo: Arc<dyn ExecutionRepository>,
        react_engine: Arc<ReActEngine>,
        ai_client: Arc<dyn AiClient>,
    ) -> Self {
        Self {
            agent_repo,
            execution_repo,
            react_engine,
            ai_client,
        }
    }

    /// エージェントを取得し、ReActエンジンで実行する
    pub async fn execute(&self, req: ExecuteAgentRequest) -> anyhow::Result<ExecuteAgentResponse> {
        // エージェント定義を取得する
        let agent = self
            .agent_repo
            .find_by_id(&req.agent_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Agent not found: {}", req.agent_id))?;

        info!(agent_id = %req.agent_id, "starting agent execution");

        let now = Utc::now();
        let execution_id = Uuid::new_v4().to_string();

        // 実行レコードを作成する
        let mut execution = Execution {
            id: execution_id,
            agent_id: req.agent_id,
            session_id: req.session_id,
            tenant_id: req.tenant_id,
            input: req.input.clone(),
            output: None,
            status: ExecutionStatus::Running,
            steps: Vec::new(),
            created_at: now,
            updated_at: now,
        };

        // 実行中の状態で保存する
        self.execution_repo.save(&execution).await?;

        // ReActエンジンで実行する
        match self
            .react_engine
            .execute(&agent, &req.input, self.ai_client.as_ref())
            .await
        {
            Ok(steps) => {
                // 最後のステップの出力を実行結果として取得する
                let output = steps.last().map(|s| s.output.clone()).unwrap_or_default();
                execution.steps = steps;
                execution.output = Some(output);
                execution.status = ExecutionStatus::Completed;
            }
            Err(e) => {
                // エラー時はFailed状態に設定する
                execution.output = Some(format!("Error: {}", e));
                execution.status = ExecutionStatus::Failed;
            }
        }

        execution.updated_at = Utc::now();

        // 完了状態で保存する
        self.execution_repo.save(&execution).await?;

        Ok(ExecuteAgentResponse { execution })
    }
}
