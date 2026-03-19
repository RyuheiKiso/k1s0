use moka::future::Cache;
use std::sync::Arc;
use std::time::Duration;

use crate::domain::entity::rule::{Rule, RuleSet};

// TODO: dead_code監査 - RuleCache は startup.rs で生成されるが _cache に代入されており実質未使用。キャッシュ統合を検討して削除を検討
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

    // TODO: dead_code監査 - 使用箇所を確認して削除を検討
    #[allow(dead_code)]
    pub async fn get_rule(&self, id: &uuid::Uuid) -> Option<Arc<Rule>> {
        self.rules.get(&id.to_string()).await
    }

    // TODO: dead_code監査 - 使用箇所を確認して削除を検討
    #[allow(dead_code)]
    pub async fn insert_rule(&self, rule: Arc<Rule>) {
        self.rules.insert(rule.id.to_string(), rule).await;
    }

    // TODO: dead_code監査 - 使用箇所を確認して削除を検討
    #[allow(dead_code)]
    pub async fn invalidate_rule(&self, id: &uuid::Uuid) {
        self.rules.invalidate(&id.to_string()).await;
    }

    // TODO: dead_code監査 - 使用箇所を確認して削除を検討
    #[allow(dead_code)]
    pub async fn get_rule_set(&self, id: &uuid::Uuid) -> Option<Arc<RuleSet>> {
        self.rule_sets.get(&id.to_string()).await
    }

    // TODO: dead_code監査 - 使用箇所を確認して削除を検討
    #[allow(dead_code)]
    pub async fn insert_rule_set(&self, rs: Arc<RuleSet>) {
        self.rule_sets.insert(rs.id.to_string(), rs).await;
    }

    // TODO: dead_code監査 - 使用箇所を確認して削除を検討
    #[allow(dead_code)]
    pub async fn invalidate_rule_set(&self, id: &uuid::Uuid) {
        self.rule_sets.invalidate(&id.to_string()).await;
    }

    // TODO: dead_code監査 - 使用箇所を確認して削除を検討
    #[allow(dead_code)]
    pub async fn invalidate_all(&self) {
        self.rules.invalidate_all();
        self.rule_sets.invalidate_all();
        self.rules.run_pending_tasks().await;
        self.rule_sets.run_pending_tasks().await;
    }
}
