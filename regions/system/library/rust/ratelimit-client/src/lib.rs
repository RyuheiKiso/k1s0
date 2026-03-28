pub mod client;
pub mod error;
pub mod grpc;
pub mod in_memory;
pub mod types;

pub use client::RateLimitClient;
pub use error::RateLimitError;
pub use in_memory::InMemoryRateLimitClient;
pub use types::{RateLimitPolicy, RateLimitResult, RateLimitStatus};

/// C-02 監査対応: GrpcRateLimitClient → HttpRateLimitClient にリネーム
#[cfg(feature = "grpc")]
pub use grpc::HttpRateLimitClient;

/// 後方互換性のための型エイリアス（L-16 監査対応: 旧名称からの移行期間用）
#[cfg(feature = "grpc")]
#[allow(deprecated)]
pub use grpc::GrpcRateLimitClient;
