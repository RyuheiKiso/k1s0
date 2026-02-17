pub mod config_grpc;
pub mod tonic_service;

pub use config_grpc::ConfigGrpcService;
pub use tonic_service::ConfigServiceTonic;
