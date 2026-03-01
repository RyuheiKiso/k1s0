pub mod client;
pub mod config;
pub mod error;
pub mod tenant;

pub use client::{HttpTenantClient, InMemoryTenantClient, TenantClient};
pub use config::TenantClientConfig;
pub use error::TenantError;
pub use tenant::{
    CreateTenantRequest, ProvisioningStatus, Tenant, TenantFilter, TenantMember, TenantSettings,
    TenantStatus,
};

#[cfg(feature = "mock")]
pub use client::MockTenantClient;
