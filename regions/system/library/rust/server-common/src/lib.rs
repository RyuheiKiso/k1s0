//! k1s0-server-common: Shared server infrastructure for k1s0 system tier.
//!
//! Provides structured error codes following the `SYS_{SERVICE}_{ERROR}` pattern,
//! unified error response types, and axum integration for HTTP error responses.

pub mod auth;
pub mod error;
pub mod infra_guard;
#[cfg(any(feature = "middleware", test))]
pub mod middleware;
pub mod pagination;
pub mod response;
/// グレースフルシャットダウン用のシグナル待機モジュール
#[cfg(feature = "shutdown")]
pub mod shutdown;

/// デフォルトの OpenTelemetry コレクターエンドポイント。
/// 全サーバーの設定デフォルト値として使用する。エンドポイント変更時はここだけ修正すればよい。
pub const DEFAULT_OTEL_ENDPOINT: &str = "http://otel-collector.observability:4317";

pub use auth::{allow_insecure_no_auth, require_auth_state};
pub use infra_guard::{allow_in_memory_infra, require_infra, InfraKind};
pub use error::{ErrorBody, ErrorCode, ErrorDetail, ErrorResponse, ServiceError};
pub use pagination::{PaginatedResponse, PaginationResponse};
pub use response::ApiResponse;
