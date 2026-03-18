// インメモリリポジトリ
// テスト・開発用のメモリ内リポジトリ実装

use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::domain::entity::{AgentDefinition, Execution};
use crate::domain::repository::{AgentRepository, ExecutionRepository};

/// InMemoryAgentRepository はメモリ内のエージェントリポジトリ
pub struct InMemoryAgentRepository {
    /// エージェント定義のストア
    agents: Arc<RwLock<Vec<AgentDefinition>>>,
}

impl InMemoryAgentRepository {
    /// 新しいInMemoryAgentRepositoryを生成する
    pub fn new() -> Self {
        Self {
            agents: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

impl Default for InMemoryAgentRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AgentRepository for InMemoryAgentRepository {
    /// IDでエージェント定義を検索する
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<AgentDefinition>> {
        let agents = self.agents.read().await;
        Ok(agents.iter().find(|a| a.id == id).cloned())
    }

    /// すべてのエージェント定義を取得する
    async fn find_all(&self) -> anyhow::Result<Vec<AgentDefinition>> {
        let agents = self.agents.read().await;
        Ok(agents.clone())
    }

    /// エージェント定義を保存する（既存の場合は更新）
    async fn save(&self, agent: &AgentDefinition) -> anyhow::Result<()> {
        let mut agents = self.agents.write().await;
        if let Some(pos) = agents.iter().position(|a| a.id == agent.id) {
            agents[pos] = agent.clone();
        } else {
            agents.push(agent.clone());
        }
        Ok(())
    }
}

/// InMemoryExecutionRepository はメモリ内の実行リポジトリ
pub struct InMemoryExecutionRepository {
    /// 実行のストア
    executions: Arc<RwLock<Vec<Execution>>>,
}

impl InMemoryExecutionRepository {
    /// 新しいInMemoryExecutionRepositoryを生成する
    pub fn new() -> Self {
        Self {
            executions: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

impl Default for InMemoryExecutionRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ExecutionRepository for InMemoryExecutionRepository {
    /// IDで実行を検索する
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<Execution>> {
        let executions = self.executions.read().await;
        Ok(executions.iter().find(|e| e.id == id).cloned())
    }

    /// 実行を保存する（既存の場合は更新）
    async fn save(&self, execution: &Execution) -> anyhow::Result<()> {
        let mut executions = self.executions.write().await;
        if let Some(pos) = executions.iter().position(|e| e.id == execution.id) {
            executions[pos] = execution.clone();
        } else {
            executions.push(execution.clone());
        }
        Ok(())
    }

    /// エージェントIDで実行履歴を取得する
    async fn find_by_agent(&self, agent_id: &str) -> anyhow::Result<Vec<Execution>> {
        let executions = self.executions.read().await;
        Ok(executions
            .iter()
            .filter(|e| e.agent_id == agent_id)
            .cloned()
            .collect())
    }
}
