// エージェント定義PostgreSQLリポジトリ
// sqlxを使用してPostgreSQLからAgentDefinitionを読み書きする

use std::sync::Arc;

use async_trait::async_trait;
use sqlx::PgPool;

use crate::domain::entity::AgentDefinition;
use crate::domain::repository::AgentRepository;

/// `AgentPostgresRepository` `はPostgreSQLベースのエージェントリポジトリ実装`
pub struct AgentPostgresRepository {
    /// データベースコネクションプール
    pool: Arc<PgPool>,
}

impl AgentPostgresRepository {
    /// `新しいAgentPostgresRepositoryを生成する`
    #[must_use] 
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AgentRepository for AgentPostgresRepository {
    /// IDでエージェント定義を検索する
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<AgentDefinition>> {
        let row = sqlx::query_as::<_, AgentRow>(
            "SELECT id, name, description, model_id, system_prompt, tools, max_steps, enabled, created_at, updated_at FROM agent_definitions WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(self.pool.as_ref())
        .await?;

        Ok(row.map(std::convert::Into::into))
    }

    /// すべてのエージェント定義を取得する
    async fn find_all(&self) -> anyhow::Result<Vec<AgentDefinition>> {
        let rows = sqlx::query_as::<_, AgentRow>(
            "SELECT id, name, description, model_id, system_prompt, tools, max_steps, enabled, created_at, updated_at FROM agent_definitions ORDER BY created_at DESC"
        )
        .fetch_all(self.pool.as_ref())
        .await?;

        Ok(rows.into_iter().map(std::convert::Into::into).collect())
    }

    /// エージェント定義をUPSERTで保存する
    async fn save(&self, agent: &AgentDefinition) -> anyhow::Result<()> {
        let tools_json = serde_json::to_value(&agent.tools)?;
        sqlx::query(
            "INSERT INTO agent_definitions (id, name, description, model_id, system_prompt, tools, max_steps, enabled, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) \
             ON CONFLICT (id) DO UPDATE SET name = $2, description = $3, model_id = $4, system_prompt = $5, tools = $6, max_steps = $7, enabled = $8, updated_at = $10"
        )
        .bind(&agent.id)
        .bind(&agent.name)
        .bind(&agent.description)
        .bind(&agent.model_id)
        .bind(&agent.system_prompt)
        .bind(&tools_json)
        .bind(agent.max_steps)
        .bind(agent.enabled)
        .bind(agent.created_at)
        .bind(agent.updated_at)
        .execute(self.pool.as_ref())
        .await?;

        Ok(())
    }
}

/// `データベース行からAgentDefinitionへのマッピング用構造体`
#[derive(sqlx::FromRow)]
struct AgentRow {
    id: String,
    name: String,
    description: String,
    model_id: String,
    system_prompt: String,
    tools: serde_json::Value,
    max_steps: i32,
    enabled: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<AgentRow> for AgentDefinition {
    fn from(row: AgentRow) -> Self {
        let tools: Vec<String> = serde_json::from_value(row.tools).unwrap_or_default();
        Self {
            id: row.id,
            name: row.name,
            description: row.description,
            model_id: row.model_id,
            system_prompt: row.system_prompt,
            tools,
            max_steps: row.max_steps,
            enabled: row.enabled,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
