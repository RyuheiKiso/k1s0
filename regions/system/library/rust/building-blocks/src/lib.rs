// Re-export core types (backward compatible with existing imports)
pub mod component {
    pub use k1s0_bb_core::component::*;
}

pub mod config {
    pub use k1s0_bb_core::config::*;
}

pub mod error {
    pub use k1s0_bb_core::error::*;
}

pub mod registry {
    pub use k1s0_bb_core::registry::*;
}

// Re-export sub-crate types for unified access
pub mod binding {
    pub use k1s0_bb_binding::*;
}

pub mod pubsub {
    pub use k1s0_bb_pubsub::*;
}

pub mod secretstore {
    pub use k1s0_bb_secretstore::*;
}

pub mod statestore {
    pub use k1s0_bb_statestore::*;
}

// Top-level re-exports
pub use component::{Component, ComponentStatus};
pub use config::{ComponentConfig, ComponentsConfig};
pub use error::ComponentError;
pub use registry::ComponentRegistry;
