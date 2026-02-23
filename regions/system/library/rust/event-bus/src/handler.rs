use async_trait::async_trait;

use crate::error::EventBusError;
use crate::event::Event;

#[async_trait]
#[cfg_attr(feature = "mock", mockall::automock)]
pub trait EventHandler: Send + Sync {
    fn event_type(&self) -> &str;
    async fn handle(&self, event: Event) -> Result<(), EventBusError>;
}
