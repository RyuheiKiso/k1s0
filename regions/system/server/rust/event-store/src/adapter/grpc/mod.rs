pub mod event_store_grpc;
pub mod tonic_service;

pub use event_store_grpc::EventStoreGrpcService;
pub use tonic_service::EventStoreServiceTonic;
