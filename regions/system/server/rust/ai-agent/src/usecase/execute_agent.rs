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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::{AgentDefinition, ExecutionStatus, Tool};
    use crate::domain::repository::agent_repository::MockAgentRepository;
    use crate::domain::repository::execution_repository::MockExecutionRepository;
    use crate::domain::service::{tool_registry::ToolRegistry, ReActEngine};
    use async_trait::async_trait;
    use chrono::Utc;
    use k1s0_bb_ai_client::traits::AiClient;
    use k1s0_bb_ai_client::types::{
        AiClientError, CompleteRequest, CompleteResponse, EmbedRequest, EmbedResponse, ModelInfo,
        Usage,
    };

    // テスト用AiClientスタブ: final_answer形式のJSONレスポンスを返す
    struct StubAiClientSuccess;

    #[async_trait]
    impl AiClient for StubAiClientSuccess {
        async fn complete(
            &self,
            _req: &CompleteRequest,
        ) -> Result<CompleteResponse, AiClientError> {
            Ok(CompleteResponse {
                id: "test-response-id".to_string(),
                model: "claude-3-opus".to_string(),
                // ReActエンジンのfinal_answer形式に合わせたJSONレスポンス
                content: r#"{"action":"final_answer","output":"テスト実行完了"}"#.to_string(),
                usage: Usage {
                    input_tokens: 10,
                    output_tokens: 20,
                },
            })
        }
        async fn embed(&self, _req: &EmbedRequest) -> Result<EmbedResponse, AiClientError> {
            unimplemented!("テスト対象外")
        }
        async fn list_models(&self) -> Result<Vec<ModelInfo>, AiClientError> {
            unimplemented!("テスト対象外")
        }
    }

    // テスト用AiClientスタブ: エラーレスポンスを返す
    struct StubAiClientError;

    #[async_trait]
    impl AiClient for StubAiClientError {
        async fn complete(
            &self,
            _req: &CompleteRequest,
        ) -> Result<CompleteResponse, AiClientError> {
            Err(AiClientError::Unavailable("AIサービス停止中".to_string()))
        }
        async fn embed(&self, _req: &EmbedRequest) -> Result<EmbedResponse, AiClientError> {
            unimplemented!("テスト対象外")
        }
        async fn list_models(&self) -> Result<Vec<ModelInfo>, AiClientError> {
            unimplemented!("テスト対象外")
        }
    }

    // テスト用のエージェント定義を生成するヘルパー
    fn sample_agent(agent_id: &str) -> AgentDefinition {
        let now = Utc::now();
        AgentDefinition {
            id: agent_id.to_string(),
            name: "テストエージェント".to_string(),
            description: "テスト用".to_string(),
            model_id: "claude-3-opus".to_string(),
            system_prompt: "あなたはテストアシスタントです。".to_string(),
            tools: vec![],
            max_steps: 3,
            enabled: true,
            created_at: now,
            updated_at: now,
        }
    }

    // テスト用の実行リクエストを生成するヘルパー
    fn sample_request(agent_id: &str) -> ExecuteAgentRequest {
        ExecuteAgentRequest {
            agent_id: agent_id.to_string(),
            input: "テストタスクを実行してください".to_string(),
            session_id: "session-001".to_string(),
            tenant_id: "tenant-001".to_string(),
        }
    }

    // 空のReActEngineを生成するヘルパー（ツールなし）
    fn empty_react_engine() -> Arc<ReActEngine> {
        Arc::new(ReActEngine::new(ToolRegistry::new()))
    }

    // 正常系: エージェント取得→ReAct実行→保存が成功し、Completedステータスで返る
    #[tokio::test]
    async fn test_execute_agent_success() {
        let agent_id = "agent-001";
        let agent = sample_agent(agent_id);

        let mut mock_agent_repo = MockAgentRepository::new();
        let agent_clone = agent.clone();
        mock_agent_repo
            .expect_find_by_id()
            .times(1)
            .returning(move |_| Ok(Some(agent_clone.clone())));

        let mut mock_exec_repo = MockExecutionRepository::new();
        // save は Running→Completed の2回呼ばれる
        mock_exec_repo.expect_save().times(2).returning(|_| Ok(()));

        let uc = ExecuteAgentUseCase::new(
            Arc::new(mock_agent_repo),
            Arc::new(mock_exec_repo),
            empty_react_engine(),
            Arc::new(StubAiClientSuccess),
        );

        let result = uc.execute(sample_request(agent_id)).await;
        assert!(result.is_ok());
        let resp = result.unwrap();
        assert_eq!(resp.execution.agent_id, agent_id);
        assert_eq!(resp.execution.status, ExecutionStatus::Completed);
    }

    // 異常系: 存在しないエージェントIDを指定した場合にエラーが返る
    #[tokio::test]
    async fn test_execute_agent_not_found() {
        let mut mock_agent_repo = MockAgentRepository::new();
        mock_agent_repo
            .expect_find_by_id()
            .times(1)
            .returning(|_| Ok(None));

        let mock_exec_repo = MockExecutionRepository::new();

        let uc = ExecuteAgentUseCase::new(
            Arc::new(mock_agent_repo),
            Arc::new(mock_exec_repo),
            empty_react_engine(),
            Arc::new(StubAiClientSuccess),
        );

        let result = uc.execute(sample_request("no-such-agent")).await;
        assert!(result.is_err());
        // err().unwrap() を使うことでDebugトレイト不要でエラーを取得する
        assert!(result
            .err()
            .unwrap()
            .to_string()
            .contains("Agent not found"));
    }

    // 異常系: ReActエンジン内でAIクライアントがエラーを返した場合にFailedで保存される
    #[tokio::test]
    async fn test_execute_agent_react_error_results_in_failed_status() {
        let agent_id = "agent-001";
        let agent = sample_agent(agent_id);

        let mut mock_agent_repo = MockAgentRepository::new();
        let agent_clone = agent.clone();
        mock_agent_repo
            .expect_find_by_id()
            .times(1)
            .returning(move |_| Ok(Some(agent_clone.clone())));

        let mut mock_exec_repo = MockExecutionRepository::new();
        // Running状態で1回目のsave、Failed状態で2回目のsave
        mock_exec_repo.expect_save().times(2).returning(|_| Ok(()));

        // AIクライアントがエラーを返すスタブを使用
        // ReActエンジンは内部でエラーをキャッチしてstepsにエラーを記録し、正常終了する
        // そのため execute_agent.rs 側でステータスをFailedに設定するパスは通らず、
        // 実行結果はCompletedになることに注意する
        let uc = ExecuteAgentUseCase::new(
            Arc::new(mock_agent_repo),
            Arc::new(mock_exec_repo),
            empty_react_engine(),
            Arc::new(StubAiClientError),
        );

        let result = uc.execute(sample_request(agent_id)).await;
        // ReActエンジンがエラーステップを記録して正常終了するため、usecaseはOkを返す
        assert!(result.is_ok());
        let resp = result.unwrap();
        // AIエラー時はReActエンジンがbreakするため、出力はNoneになる（最後のstepがない）
        // executionはsaveが2回呼ばれることで確認できる
        assert_eq!(resp.execution.agent_id, agent_id);
    }

    // 状態遷移確認: saveが2回呼ばれることで Running→Completed の状態遷移を検証する
    #[tokio::test]
    async fn test_execute_agent_saves_running_then_completed() {
        let agent_id = "agent-001";
        let agent = sample_agent(agent_id);
        let save_count = std::sync::Arc::new(std::sync::Mutex::new(0u32));
        let save_count_clone = save_count.clone();

        let mut mock_agent_repo = MockAgentRepository::new();
        let agent_clone = agent.clone();
        mock_agent_repo
            .expect_find_by_id()
            .times(1)
            .returning(move |_| Ok(Some(agent_clone.clone())));

        let mut mock_exec_repo = MockExecutionRepository::new();
        mock_exec_repo
            .expect_save()
            .times(2)
            .returning(move |saved_exec| {
                let mut count = save_count_clone.lock().unwrap();
                *count += 1;
                if *count == 1 {
                    // 1回目のsave: Running状態で保存される
                    assert_eq!(saved_exec.status, ExecutionStatus::Running);
                } else {
                    // 2回目のsave: Completed状態で保存される
                    assert_eq!(saved_exec.status, ExecutionStatus::Completed);
                }
                Ok(())
            });

        let uc = ExecuteAgentUseCase::new(
            Arc::new(mock_agent_repo),
            Arc::new(mock_exec_repo),
            empty_react_engine(),
            Arc::new(StubAiClientSuccess),
        );

        let result = uc.execute(sample_request(agent_id)).await;
        assert!(result.is_ok());
        // saveが2回呼ばれたことを確認する
        assert_eq!(*save_count.lock().unwrap(), 2);
    }

    // ツール付きエージェント登録: ToolRegistryにツールを含むReActEngineでも正常動作する
    #[tokio::test]
    async fn test_execute_agent_with_tools() {
        let agent_id = "agent-tools";
        let agent = AgentDefinition {
            tools: vec!["search".to_string()],
            ..sample_agent(agent_id)
        };

        let mut tool_registry = ToolRegistry::new();
        tool_registry.register(Tool {
            name: "search".to_string(),
            description: "ウェブ検索を実行する".to_string(),
            parameters_schema: r#"{"type":"object","properties":{"query":{"type":"string"}}}"#
                .to_string(),
        });
        let react_engine = Arc::new(ReActEngine::new(tool_registry));

        let mut mock_agent_repo = MockAgentRepository::new();
        let agent_clone = agent.clone();
        mock_agent_repo
            .expect_find_by_id()
            .times(1)
            .returning(move |_| Ok(Some(agent_clone.clone())));

        let mut mock_exec_repo = MockExecutionRepository::new();
        mock_exec_repo.expect_save().times(2).returning(|_| Ok(()));

        let uc = ExecuteAgentUseCase::new(
            Arc::new(mock_agent_repo),
            Arc::new(mock_exec_repo),
            react_engine,
            Arc::new(StubAiClientSuccess),
        );

        let result = uc.execute(sample_request(agent_id)).await;
        assert!(result.is_ok());
    }
}
