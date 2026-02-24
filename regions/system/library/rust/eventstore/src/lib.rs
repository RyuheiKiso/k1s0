pub mod envelope;
pub mod error;
pub mod memory;
#[cfg(feature = "postgres")]
pub mod postgres;
pub mod snapshot;
pub mod store;
pub mod stream;

pub use envelope::EventEnvelope;
pub use error::EventStoreError;
pub use memory::{InMemoryEventStore, InMemorySnapshotStore};
#[cfg(feature = "postgres")]
pub use self::postgres::PostgresEventStore;
pub use snapshot::{Snapshot, SnapshotStore};
pub use store::EventStore;
pub use stream::StreamId;

#[cfg(feature = "mock")]
pub use store::MockEventStore;
