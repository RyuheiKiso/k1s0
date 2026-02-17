pub mod auth_grpc;
pub mod audit_grpc;
pub mod tonic_service;

pub use auth_grpc::AuthGrpcService;
pub use audit_grpc::AuditGrpcService;
pub use tonic_service::{AuthServiceTonic, AuditServiceTonic};
