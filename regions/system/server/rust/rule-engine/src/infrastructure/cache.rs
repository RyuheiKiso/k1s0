use moka::future::Cache;
use std::sync::Arc;
use std::time::Duration;

use crate::domain::entity::rule::{Rule, RuleSet};

// RuleCache はルール評価のパフォーマンス最適化のためのキャッシュ層。
// startup.rs で生成済みだがリポジトリ層との統合が未完了のため dead_code を許可。
#[allow(dead_code)]
pub struct RuleCache {
    rules: Cache<String, Arc<Rule>>,
    rule_sets: Cache<String, Arc<RuleSet>>,
}

impl RuleCache {
    pub fn new(max_capacity: u64, ttl_secs: u64) -> Self {
        let ttl = Duration::from_secs(ttl_secs);
        let rules = Cache::builder()
            .max_capacity(max_capacity)
            .time_to_live(ttl)
            .build();
        let rule_sets = Cache::builder()
            .max_capacity(max_capacity)
            .time_to_live(ttl)
            .build();
        Self { rules, rule_sets }
    }

    // リポジトリ層との統合時に使用予定のため dead_code を許可
    #[allow(dead_code)]
    pub async fn get_rule(&self, id: &uuid::Uuid) -> Option<Arc<Rule>> {
        self.rules.get(&id.to_string()).await
    }

    // リポジトリ層との統合時に使用予定のため dead_code を許可
    #[allow(dead_code)]
    pub async fn insert_rule(&self, rule: Arc<Rule>) {
        self.rules.insert(rule.id.to_string(), rule).await;
    }

    // リポジトリ層との統合時に使用予定のため dead_code を許可
    #[allow(dead_code)]
    pub async fn invalidate_rule(&self, id: &uuid::Uuid) {
        self.rules.invalidate(&id.to_string()).await;
    }

    // リポジトリ層との統合時に使用予定のため dead_code を許可
    #[allow(dead_code)]
    pub async fn get_rule_set(&self, id: &uuid::Uuid) -> Option<Arc<RuleSet>> {
        self.rule_sets.get(&id.to_string()).await
    }

    // リポジトリ層との統合時に使用予定のため dead_code を許可
    #[allow(dead_code)]
    pub async fn insert_rule_set(&self, rs: Arc<RuleSet>) {
        self.rule_sets.insert(rs.id.to_string(), rs).await;
    }

    // リポジトリ層との統合時に使用予定のため dead_code を許可
    #[allow(dead_code)]
    pub async fn invalidate_rule_set(&self, id: &uuid::Uuid) {
        self.rule_sets.invalidate(&id.to_string()).await;
    }

    // リポジトリ層との統合時に使用予定のため dead_code を許可
    #[allow(dead_code)]
    pub async fn invalidate_all(&self) {
        self.rules.invalidate_all();
        self.rule_sets.invalidate_all();
        self.rules.run_pending_tasks().await;
        self.rule_sets.run_pending_tasks().await;
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::entity::rule::{EvaluationMode, Rule, RuleSet};
    use uuid::Uuid;

    fn make_rule(name: &str) -> Rule {
        Rule::new(
            name.to_string(),
            "test".to_string(),
            1,
            serde_json::json!({"field": "x", "operator": "eq", "value": "y"}),
            serde_json::json!({}),
        )
    }

    fn make_rule_set(name: &str) -> RuleSet {
        RuleSet::new(
            name.to_string(),
            "test".to_string(),
            "domain".to_string(),
            EvaluationMode::FirstMatch,
            serde_json::json!({}),
            vec![],
        )
    }

    /// ルールを挿入した後に取得できる
    #[tokio::test]
    async fn insert_and_get_rule() {
        let cache = RuleCache::new(100, 60);
        let rule = make_rule("test-rule");
        let id = rule.id;
        cache.insert_rule(Arc::new(rule)).await;
        let result = cache.get_rule(&id).await;
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "test-rule");
    }

    /// 存在しないルールを取得すると None を返す
    #[tokio::test]
    async fn get_nonexistent_rule_returns_none() {
        let cache = RuleCache::new(100, 60);
        let result = cache.get_rule(&Uuid::new_v4()).await;
        assert!(result.is_none());
    }

    /// ルールを無効化した後は取得できない
    #[tokio::test]
    async fn invalidate_rule_removes_entry() {
        let cache = RuleCache::new(100, 60);
        let rule = make_rule("evict-me");
        let id = rule.id;
        cache.insert_rule(Arc::new(rule)).await;
        cache.invalidate_rule(&id).await;
        // invalidate後はrun_pending_tasksが必要なケースがあるが、通常は即時反映
        // ここでは invalidate_all を使って確実に除去
        cache.rules.run_pending_tasks().await;
        let result = cache.get_rule(&id).await;
        assert!(result.is_none());
    }

    /// RuleSetを挿入した後に取得できる
    #[tokio::test]
    async fn insert_and_get_rule_set() {
        let cache = RuleCache::new(100, 60);
        let rs = make_rule_set("test-set");
        let id = rs.id;
        cache.insert_rule_set(Arc::new(rs)).await;
        let result = cache.get_rule_set(&id).await;
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "test-set");
    }

    /// invalidate_all でルールとRuleSetの両方が除去される
    #[tokio::test]
    async fn invalidate_all_clears_both_caches() {
        let cache = RuleCache::new(100, 60);
        let rule = make_rule("r1");
        let rule_id = rule.id;
        let rs = make_rule_set("rs1");
        let rs_id = rs.id;
        cache.insert_rule(Arc::new(rule)).await;
        cache.insert_rule_set(Arc::new(rs)).await;
        cache.invalidate_all().await;
        assert!(cache.get_rule(&rule_id).await.is_none());
        assert!(cache.get_rule_set(&rs_id).await.is_none());
    }
}
