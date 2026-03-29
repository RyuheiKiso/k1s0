mod app;
#[cfg(feature = "auth")]
pub mod auth_middleware;
#[cfg(feature = "config-loader")]
pub mod config_loader;
#[cfg(feature = "database")]
pub mod database;
#[cfg(feature = "grpc-auth")]
pub mod grpc_auth;
#[cfg(feature = "kafka-setup")]
pub mod kafka_setup;
#[cfg(feature = "auth")]
pub mod rbac;
mod request_id;
/// shutdown フィーチャーが有効な場合のみ、シャットダウンシグナルの再エクスポートを提供する
#[cfg(feature = "shutdown")]
pub mod shutdown;
mod stack;
mod standard_routes;

#[cfg(feature = "auth")]
pub use app::{AuthConfig, JwksConfig, JwtConfig};
pub use app::{K1s0App, K1s0AppReady};
#[cfg(feature = "auth")]
pub use auth_middleware::AuthState;
#[cfg(feature = "config-loader")]
pub use config_loader::{load_config, ConfigError};
#[cfg(feature = "database")]
pub use database::{DatabaseConfig, DatabaseSetup};
#[cfg(feature = "grpc-auth")]
pub use grpc_auth::GrpcAuthLayer;
#[cfg(feature = "kafka-setup")]
pub use kafka_setup::{KafkaConfig, KafkaSetup};
#[cfg(feature = "auth")]
pub use rbac::{check_permission, Tier};
pub use request_id::RequestIdLayer;
pub use stack::{K1s0Stack, Profile};
