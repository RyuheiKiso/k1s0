//! k1s0-server-common: Shared server infrastructure for k1s0 system tier.
//!
//! Provides structured error codes following the `SYS_{SERVICE}_{ERROR}` pattern,
//! unified error response types, and axum integration for HTTP error responses.

pub mod auth;
pub mod error;
pub mod pagination;

pub use auth::{allow_insecure_no_auth, require_auth_state};
pub use error::{ErrorBody, ErrorCode, ErrorDetail, ErrorResponse, ServiceError};
pub use pagination::{PaginatedResponse, PaginationResponse};
