// エージェント作成ユースケース
// 新しいエージェント定義をリポジトリに保存する

use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use crate::domain::entity::AgentDefinition;
use crate::domain::repository::AgentRepository;

/// `CreateAgentUseCase` はエージェント定義の作成を担当する
pub struct CreateAgentUseCase {
    /// エージェントリポジトリ
    agent_repo: Arc<dyn AgentRepository>,
}

/// エージェント作成リクエスト
pub struct CreateAgentRequest {
    pub name: String,
    pub description: String,
    pub model_id: String,
    pub system_prompt: String,
    pub tools: Vec<String>,
    pub max_steps: i32,
}

/// エージェント作成レスポンス
pub struct CreateAgentResponse {
    pub agent: AgentDefinition,
}

impl CreateAgentUseCase {
    /// `新しいCreateAgentUseCaseを生成する`
    pub fn new(agent_repo: Arc<dyn AgentRepository>) -> Self {
        Self { agent_repo }
    }

    /// エージェント定義を作成して保存する
    pub async fn execute(&self, req: CreateAgentRequest) -> anyhow::Result<CreateAgentResponse> {
        let now = Utc::now();
        let agent = AgentDefinition {
            id: Uuid::new_v4().to_string(),
            name: req.name,
            description: req.description,
            model_id: req.model_id,
            system_prompt: req.system_prompt,
            tools: req.tools,
            max_steps: req.max_steps,
            enabled: true,
            created_at: now,
            updated_at: now,
        };

        // リポジトリに保存する
        self.agent_repo.save(&agent).await?;

        Ok(CreateAgentResponse { agent })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::agent_repository::MockAgentRepository;

    // テスト用エージェント作成リクエストを生成するヘルパー関数
    fn sample_request() -> CreateAgentRequest {
        CreateAgentRequest {
            name: "テストエージェント".to_string(),
            description: "テスト用エージェント定義".to_string(),
            model_id: "claude-3-opus".to_string(),
            system_prompt: "あなたはテスト用のAIアシスタントです。".to_string(),
            tools: vec!["search".to_string(), "calculator".to_string()],
            max_steps: 5,
        }
    }

    // 正常系: リポジトリへの保存が成功し、レスポンスにエージェントが含まれる
    #[tokio::test]
    async fn test_create_agent_success() {
        let mut mock_repo = MockAgentRepository::new();
        // save が1回呼ばれることを期待する
        mock_repo.expect_save().times(1).returning(|_| Ok(()));

        let uc = CreateAgentUseCase::new(Arc::new(mock_repo));
        let result = uc.execute(sample_request()).await;

        assert!(result.is_ok());
        let resp = result.unwrap();
        assert_eq!(resp.agent.name, "テストエージェント");
        assert_eq!(resp.agent.model_id, "claude-3-opus");
        assert_eq!(resp.agent.max_steps, 5);
        assert_eq!(resp.agent.tools.len(), 2);
    }

    // 異常系: リポジトリのsaveが失敗した場合にエラーが伝播する
    #[tokio::test]
    async fn test_create_agent_repo_error() {
        let mut mock_repo = MockAgentRepository::new();
        mock_repo
            .expect_save()
            .times(1)
            .returning(|_| Err(anyhow::anyhow!("データベース接続エラー")));

        let uc = CreateAgentUseCase::new(Arc::new(mock_repo));
        let result = uc.execute(sample_request()).await;

        assert!(result.is_err());
        // err().unwrap() を使うことでDebugトレイト不要でエラーを取得する
        assert!(result
            .err()
            .unwrap()
            .to_string()
            .contains("データベース接続エラー"));
    }

    // フィールド自動設定: id, enabled, created_at, updated_at が正しく設定される
    #[tokio::test]
    async fn test_create_agent_auto_fields_populated() {
        let mut mock_repo = MockAgentRepository::new();
        mock_repo.expect_save().times(1).returning(|_| Ok(()));

        let uc = CreateAgentUseCase::new(Arc::new(mock_repo));
        let result = uc.execute(sample_request()).await.unwrap();
        let agent = result.agent;

        // IDはUUID形式（36文字）で自動生成される
        assert_eq!(agent.id.len(), 36);
        // 新規作成時はenabled=trueに設定される
        assert!(agent.enabled);
        // created_atとupdated_atが同一タイムスタンプで設定される
        assert_eq!(agent.created_at, agent.updated_at);
    }
}
