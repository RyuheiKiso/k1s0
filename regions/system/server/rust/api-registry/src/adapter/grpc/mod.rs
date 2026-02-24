pub mod apiregistry_grpc;
pub mod tonic_service;

pub use apiregistry_grpc::ApiRegistryGrpcService;
pub use tonic_service::ApiRegistryServiceTonic;
