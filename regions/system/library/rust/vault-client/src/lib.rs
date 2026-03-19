pub mod client;
pub mod config;
pub mod error;
#[cfg(feature = "grpc")]
pub mod grpc;
pub mod http;
#[cfg(feature = "kafka")]
pub mod kafka;
pub mod secret;
pub mod startup;

pub use client::{InMemoryVaultClient, VaultClient};
pub use config::VaultClientConfig;
pub use error::VaultError;
#[cfg(feature = "grpc")]
pub use grpc::GrpcVaultClient;
pub use http::HttpVaultClient;
#[cfg(feature = "kafka")]
pub use kafka::VaultSecretRotationSubscriber;
pub use secret::{Secret, SecretRotatedEvent};
pub use startup::fetch_secrets_with_fallback;

#[cfg(feature = "mock")]
pub use client::MockVaultClient;
