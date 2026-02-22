pub mod audit_grpc;
pub mod auth_grpc;
pub mod tonic_service;

pub use audit_grpc::AuditGrpcService;
pub use auth_grpc::AuthGrpcService;
pub use tonic_service::{AuditServiceTonic, AuthServiceTonic};
