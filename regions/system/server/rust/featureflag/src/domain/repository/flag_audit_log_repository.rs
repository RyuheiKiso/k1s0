use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entity::flag_audit_log::FlagAuditLog;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait FlagAuditLogRepository: Send + Sync {
    async fn create(&self, log: &FlagAuditLog) -> anyhow::Result<()>;

    async fn list_by_flag_id(
        &self,
        flag_id: &Uuid,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<FlagAuditLog>>;
}
