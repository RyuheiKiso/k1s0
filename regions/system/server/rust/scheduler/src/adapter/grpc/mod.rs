pub mod scheduler_grpc;
pub mod tonic_service;

pub use scheduler_grpc::SchedulerGrpcService;
pub use tonic_service::SchedulerServiceTonic;
