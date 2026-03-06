pub mod event_monitor_grpc;
pub mod tonic_service;

pub use event_monitor_grpc::EventMonitorGrpcService;
pub use tonic_service::EventMonitorServiceTonic;
