// infrastructure モジュール。
// grpc: gRPC バックエンドクライアント群
// http: REST バックエンドクライアント群（service-catalog 等 gRPC 未対応サービス向け）

pub mod auth;
pub mod circuit_breaker;
pub mod config;
pub mod grpc;
pub mod grpc_retry;
pub mod http;
pub mod startup;
