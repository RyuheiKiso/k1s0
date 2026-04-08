// エージェント定義リポジトリトレイト
// エージェント定義の永続化インターフェースを定義する

use async_trait::async_trait;

use crate::domain::entity::AgentDefinition;

/// `AgentRepository` はエージェント定義の永続化を抽象化するトレイト
// テスト時にmockallによるモック自動生成を有効にする
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait AgentRepository: Send + Sync {
    /// IDでエージェント定義を検索する
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<AgentDefinition>>;

    /// すべてのエージェント定義を取得する
    #[allow(dead_code)]
    async fn find_all(&self) -> anyhow::Result<Vec<AgentDefinition>>;

    /// エージェント定義を保存する
    async fn save(&self, agent: &AgentDefinition) -> anyhow::Result<()>;
}
