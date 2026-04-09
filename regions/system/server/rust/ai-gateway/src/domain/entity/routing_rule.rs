// ルーティングルールのエンティティ定義。
// リクエストを適切なモデルに振り分けるための優先度と戦略を管理する。

use serde::{Deserialize, Serialize};

/// ルーティングルールを表すエンティティ。
/// モデル選択の優先度と戦略（コスト最適化、レイテンシ最適化など）を定義する。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRule {
    /// ルールの一意識別子
    pub id: String,
    /// 対象モデルID
    pub model_id: String,
    /// 優先度（数値が小さいほど高優先）
    pub priority: i32,
    /// ルーティング戦略（例: "cost", "latency", "round-robin"）
    pub strategy: String,
}

impl RoutingRule {
    /// 新しいルーティングルールインスタンスを生成する。
    #[must_use]
    pub fn new(id: String, model_id: String, priority: i32, strategy: String) -> Self {
        Self {
            id,
            model_id,
            priority,
            strategy,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_routing_rule() {
        let rule = RoutingRule::new(
            "rule-1".to_string(),
            "model-1".to_string(),
            1,
            "cost".to_string(),
        );
        assert_eq!(rule.id, "rule-1");
        assert_eq!(rule.model_id, "model-1");
        assert_eq!(rule.priority, 1);
        assert_eq!(rule.strategy, "cost");
    }
}
