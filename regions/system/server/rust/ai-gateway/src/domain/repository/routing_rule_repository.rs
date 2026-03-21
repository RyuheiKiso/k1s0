// ルーティングルールリポジトリのトレイト定義。
// ルーティングルールの検索を抽象化する。

use async_trait::async_trait;

use crate::domain::entity::routing_rule::RoutingRule;

/// ルーティングルールリポジトリのインターフェース。
/// アクティブなルーティングルールの検索を提供する。
// テスト時にmockallによるモック自動生成を有効にする
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait RoutingRuleRepository: Send + Sync {
    /// 指定モデルIDに対するアクティブなルーティングルールを取得する。
    async fn find_active_rule(&self, model_id: &str) -> Option<RoutingRule>;
}
