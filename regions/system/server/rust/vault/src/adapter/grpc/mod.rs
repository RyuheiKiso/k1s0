pub mod tonic_service;
pub mod vault_grpc;

pub use tonic_service::VaultServiceTonic;
pub use vault_grpc::VaultGrpcService;
