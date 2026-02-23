pub mod client;
pub mod context;
pub mod error;
pub mod flag;
pub mod memory;

pub use client::{EvaluationResult, FeatureFlagClient};
pub use context::EvaluationContext;
pub use error::FeatureFlagError;
pub use flag::{FeatureFlag, FlagVariant};
pub use memory::InMemoryFeatureFlagClient;

#[cfg(feature = "mock")]
pub use client::MockFeatureFlagClient;
