pub mod client;
pub mod config;
pub mod error;
pub mod http;
pub mod secret;
#[cfg(feature = "grpc")]
pub mod grpc;
#[cfg(feature = "kafka")]
pub mod kafka;

pub use client::{InMemoryVaultClient, VaultClient};
pub use config::VaultClientConfig;
pub use error::VaultError;
pub use http::HttpVaultClient;
pub use secret::{Secret, SecretRotatedEvent};
#[cfg(feature = "grpc")]
pub use grpc::GrpcVaultClient;
#[cfg(feature = "kafka")]
pub use kafka::VaultSecretRotationSubscriber;

#[cfg(feature = "mock")]
pub use client::MockVaultClient;
