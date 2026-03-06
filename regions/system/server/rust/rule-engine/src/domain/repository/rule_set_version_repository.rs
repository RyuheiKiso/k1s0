use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entity::rule::RuleSetVersion;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait RuleSetVersionRepository: Send + Sync {
    async fn find_by_rule_set_id_and_version(
        &self,
        rule_set_id: &Uuid,
        version: u32,
    ) -> anyhow::Result<Option<RuleSetVersion>>;
    async fn find_latest_by_rule_set_id(
        &self,
        rule_set_id: &Uuid,
    ) -> anyhow::Result<Option<RuleSetVersion>>;
    async fn create(&self, version: &RuleSetVersion) -> anyhow::Result<()>;
}
