pub mod client;
pub mod config;
pub mod error;
pub mod secret;

pub use client::{InMemoryVaultClient, VaultClient};
pub use config::VaultClientConfig;
pub use error::VaultError;
pub use secret::{Secret, SecretRotatedEvent};

#[cfg(feature = "mock")]
pub use client::MockVaultClient;
