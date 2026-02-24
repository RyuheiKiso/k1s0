//! k1s0-server-common: Shared server infrastructure for k1s0 system tier.
//!
//! Provides structured error codes following the `SYS_{SERVICE}_{ERROR}` pattern,
//! unified error response types, and axum integration for HTTP error responses.

pub mod error;

pub use error::{ErrorBody, ErrorCode, ErrorDetail, ErrorResponse, ServiceError};
