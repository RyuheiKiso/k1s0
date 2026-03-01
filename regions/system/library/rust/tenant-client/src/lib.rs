pub mod client;
pub mod config;
pub mod error;
pub mod tenant;

pub use client::{GrpcTenantClient, InMemoryTenantClient, TenantClient};
pub use config::TenantClientConfig;
pub use error::TenantError;
pub use tenant::{Tenant, TenantFilter, TenantSettings, TenantStatus};

#[cfg(feature = "mock")]
pub use client::MockTenantClient;
