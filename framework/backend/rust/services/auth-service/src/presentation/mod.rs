//! プレゼンテーション層

pub mod grpc;
pub mod rest;

pub use grpc::*;
pub use rest::create_router;
