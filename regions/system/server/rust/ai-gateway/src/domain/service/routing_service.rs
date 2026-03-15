// ルーティングサービスの実装。
// コスト戦略・レイテンシ戦略に基づいてモデルを選択するドメインロジック。

use std::sync::Arc;

use crate::domain::repository::ModelRepository;
use crate::domain::repository::RoutingRuleRepository;

/// ルーティングサービス。
/// ルーティングルールとモデル情報を組み合わせて最適なモデルを選択する。
pub struct RoutingService {
    /// モデルリポジトリ
    model_repo: Arc<dyn ModelRepository>,
    /// ルーティングルールリポジトリ
    routing_rule_repo: Arc<dyn RoutingRuleRepository>,
}

impl RoutingService {
    /// 新しいルーティングサービスを生成する。
    pub fn new(
        model_repo: Arc<dyn ModelRepository>,
        routing_rule_repo: Arc<dyn RoutingRuleRepository>,
    ) -> Self {
        Self {
            model_repo,
            routing_rule_repo,
        }
    }

    /// 指定モデルIDと戦略に基づいてモデルを選択する。
    /// ルーティングルールが存在すればそのモデルを返し、
    /// 存在しなければ戦略に応じてモデルを選択する。
    pub async fn select_model(&self, model_id: &str, strategy: &str) -> Option<String> {
        // ルーティングルールが設定されている場合はそちらを優先
        if let Some(rule) = self.routing_rule_repo.find_active_rule(model_id).await {
            return Some(rule.model_id);
        }

        // 戦略に基づくフォールバック選択
        let models = self.model_repo.find_all().await;
        let enabled_models: Vec<_> = models.into_iter().filter(|m| m.enabled).collect();

        if enabled_models.is_empty() {
            return None;
        }

        match strategy {
            // コスト最適化：入力コストが最も安いモデルを選択
            "cost" => enabled_models
                .iter()
                .min_by(|a, b| {
                    a.cost_per_1k_input
                        .partial_cmp(&b.cost_per_1k_input)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|m| m.id.clone()),
            // レイテンシ最適化：コンテキストウィンドウが最大のモデルを選択
            // （大きいコンテキストウィンドウ ≒ 高性能モデル ≒ 低レイテンシの傾向）
            "latency" => enabled_models
                .iter()
                .max_by_key(|m| m.context_window)
                .map(|m| m.id.clone()),
            // デフォルト：指定モデルIDをそのまま返す
            _ => {
                if enabled_models.iter().any(|m| m.id == model_id) {
                    Some(model_id.to_string())
                } else {
                    enabled_models.first().map(|m| m.id.clone())
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::model::AiModel;
    use crate::domain::entity::routing_rule::RoutingRule;

    /// テスト用インメモリモデルリポジトリ
    struct MockModelRepo {
        models: Vec<AiModel>,
    }

    #[async_trait::async_trait]
    impl ModelRepository for MockModelRepo {
        async fn find_all(&self) -> Vec<AiModel> {
            self.models.clone()
        }
        async fn find_by_id(&self, id: &str) -> Option<AiModel> {
            self.models.iter().find(|m| m.id == id).cloned()
        }
    }

    /// テスト用インメモリルーティングルールリポジトリ
    struct MockRoutingRuleRepo {
        rule: Option<RoutingRule>,
    }

    #[async_trait::async_trait]
    impl RoutingRuleRepository for MockRoutingRuleRepo {
        async fn find_active_rule(&self, _model_id: &str) -> Option<RoutingRule> {
            self.rule.clone()
        }
    }

    #[tokio::test]
    async fn test_select_model_cost_strategy() {
        let models = vec![
            AiModel::new(
                "m1".into(),
                "expensive".into(),
                "p".into(),
                4096,
                true,
                0.10,
                0.20,
            ),
            AiModel::new(
                "m2".into(),
                "cheap".into(),
                "p".into(),
                4096,
                true,
                0.01,
                0.02,
            ),
        ];
        let svc = RoutingService::new(
            Arc::new(MockModelRepo { models }),
            Arc::new(MockRoutingRuleRepo { rule: None }),
        );
        let result = svc.select_model("any", "cost").await;
        assert_eq!(result, Some("m2".to_string()));
    }

    #[tokio::test]
    async fn test_select_model_with_routing_rule() {
        let models = vec![AiModel::new(
            "m1".into(),
            "model-1".into(),
            "p".into(),
            4096,
            true,
            0.03,
            0.06,
        )];
        let rule = RoutingRule::new("r1".into(), "m1".into(), 1, "cost".into());
        let svc = RoutingService::new(
            Arc::new(MockModelRepo { models }),
            Arc::new(MockRoutingRuleRepo { rule: Some(rule) }),
        );
        let result = svc.select_model("any", "cost").await;
        assert_eq!(result, Some("m1".to_string()));
    }
}
