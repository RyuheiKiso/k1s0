// 実行リポジトリトレイト
// エージェント実行の永続化インターフェースを定義する

use async_trait::async_trait;

use crate::domain::entity::Execution;

/// `ExecutionRepository` はエージェント実行の永続化を抽象化するトレイト
// テスト時にmockallによるモック自動生成を有効にする
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ExecutionRepository: Send + Sync {
    /// IDで実行を検索する
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<Execution>>;

    /// 実行を保存する
    async fn save(&self, execution: &Execution) -> anyhow::Result<()>;

    /// エージェントIDで実行履歴を取得する
    async fn find_by_agent(&self, agent_id: &str) -> anyhow::Result<Vec<Execution>>;
}
