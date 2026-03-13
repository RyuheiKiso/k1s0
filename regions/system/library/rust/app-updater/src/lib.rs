pub mod checksum;
pub mod client;
pub mod config;
pub mod error;
pub mod model;
pub mod version;

pub use checksum::ChecksumVerifier;
pub use client::{AppRegistryAppUpdater, AppUpdater, InMemoryAppUpdater};
pub use config::AppUpdaterConfig;
pub use error::AppUpdaterError;
pub use model::{AppVersionInfo, DownloadArtifactInfo, UpdateCheckResult, UpdateType};
pub use version::{compare_versions, determine_update_type};
