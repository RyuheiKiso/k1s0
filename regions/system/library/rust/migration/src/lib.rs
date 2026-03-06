pub mod config;
pub mod error;
pub mod model;
pub mod runner;
#[cfg(feature = "postgres")]
pub mod sqlx_runner;

#[cfg(feature = "schema-evolution")]
pub mod analyzer;
#[cfg(feature = "schema-evolution")]
pub mod declarative;
#[cfg(feature = "schema-evolution")]
pub mod diff;
#[cfg(feature = "schema-evolution")]
pub mod reverse;
#[cfg(feature = "schema-evolution")]
pub mod schema;

pub use config::MigrationConfig;
pub use error::MigrationError;
pub use model::{MigrationFile, MigrationReport, MigrationStatus, PendingMigration};
pub use runner::{InMemoryMigrationRunner, MigrationRunner};
#[cfg(feature = "postgres")]
pub use sqlx_runner::SqlxMigrationRunner;
