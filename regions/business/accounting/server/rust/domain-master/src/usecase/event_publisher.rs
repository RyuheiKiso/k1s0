use async_trait::async_trait;
use serde_json::Value;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait DomainMasterEventPublisher: Send + Sync {
    async fn publish_category_changed(&self, event: &Value) -> anyhow::Result<()>;
    async fn publish_item_changed(&self, event: &Value) -> anyhow::Result<()>;
    async fn publish_tenant_extension_changed(&self, event: &Value) -> anyhow::Result<()>;
}

pub struct NoopDomainMasterEventPublisher;

#[async_trait]
impl DomainMasterEventPublisher for NoopDomainMasterEventPublisher {
    async fn publish_category_changed(&self, _event: &Value) -> anyhow::Result<()> {
        Ok(())
    }

    async fn publish_item_changed(&self, _event: &Value) -> anyhow::Result<()> {
        Ok(())
    }

    async fn publish_tenant_extension_changed(&self, _event: &Value) -> anyhow::Result<()> {
        Ok(())
    }
}
