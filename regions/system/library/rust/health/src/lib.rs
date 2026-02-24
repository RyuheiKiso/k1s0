pub mod checker;
pub mod checks;
pub mod error;
pub mod response;

pub use checker::{CompositeHealthChecker, HealthCheck, HealthChecker};
pub use error::HealthError;
pub use response::{CheckResult, HealthResponse, HealthStatus, HealthzResponse};

#[cfg(feature = "mock")]
pub use checker::MockHealthCheck;

// Feature-gated re-exports
#[cfg(feature = "http")]
pub use checks::http::HttpHealthCheck;

#[cfg(feature = "postgres")]
pub use checks::postgres::PostgresHealthCheck;

#[cfg(feature = "redis")]
pub use checks::redis::RedisHealthCheck;

#[cfg(feature = "kafka")]
pub use checks::kafka::KafkaHealthCheck;
