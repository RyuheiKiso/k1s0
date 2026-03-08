pub mod tenant_grpc;
pub mod tonic_service;
pub mod watch_stream;

pub use tenant_grpc::TenantGrpcService;
pub use tonic_service::TenantServiceTonic;
