pub mod bus;
pub mod config;
pub mod error;
pub mod event;
pub mod handler;

pub use bus::{EventBus, EventSubscription, InMemoryEventBus};
pub use config::EventBusConfig;
pub use error::EventBusError;
pub use event::{DomainEvent, Event};
pub use handler::{DomainEventHandler, EventHandler};

#[cfg(feature = "mock")]
pub use handler::MockEventHandler;
