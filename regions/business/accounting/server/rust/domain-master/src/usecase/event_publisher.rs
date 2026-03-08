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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_noop_publisher_category_changed() {
        let publisher = NoopDomainMasterEventPublisher;
        let event = serde_json::json!({"event_type": "test"});
        let result = publisher.publish_category_changed(&event).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_noop_publisher_item_changed() {
        let publisher = NoopDomainMasterEventPublisher;
        let event = serde_json::json!({"event_type": "test"});
        let result = publisher.publish_item_changed(&event).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_noop_publisher_tenant_extension_changed() {
        let publisher = NoopDomainMasterEventPublisher;
        let event = serde_json::json!({"event_type": "test"});
        let result = publisher.publish_tenant_extension_changed(&event).await;
        assert!(result.is_ok());
    }
}
