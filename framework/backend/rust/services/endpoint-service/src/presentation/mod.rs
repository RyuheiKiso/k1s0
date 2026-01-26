//! プレゼンテーション層
//!
//! gRPC/REST サービスの実装を定義する。

pub mod grpc;
pub mod rest;

pub use grpc::*;
pub use rest::create_router;
