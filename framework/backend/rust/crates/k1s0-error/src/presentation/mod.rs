//! Presentation 層エラー
//!
//! application 層のエラーを REST（problem+json）/ gRPC（status + metadata）へ変換する。

mod grpc;
mod http;

pub use grpc::{GrpcError, GrpcStatusCode};
pub use http::{HttpError, ProblemDetails};
