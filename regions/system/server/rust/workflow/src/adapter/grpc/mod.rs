pub mod workflow_grpc;
pub mod tonic_service;

pub use workflow_grpc::WorkflowGrpcService;
pub use tonic_service::WorkflowServiceTonic;
