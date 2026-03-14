// エージェント作成ユースケース
// 新しいエージェント定義をリポジトリに保存する

use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use crate::domain::entity::AgentDefinition;
use crate::domain::repository::AgentRepository;

/// CreateAgentUseCase はエージェント定義の作成を担当する
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
    /// 新しいCreateAgentUseCaseを生成する
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
