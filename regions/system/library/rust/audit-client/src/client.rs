use async_trait::async_trait;

use crate::error::AuditError;
use crate::event::AuditEvent;

#[async_trait]
#[cfg_attr(feature = "mock", mockall::automock)]
pub trait AuditClient: Send + Sync {
    async fn record(&self, event: AuditEvent) -> Result<(), AuditError>;
    async fn flush(&self) -> Result<Vec<AuditEvent>, AuditError>;
}
