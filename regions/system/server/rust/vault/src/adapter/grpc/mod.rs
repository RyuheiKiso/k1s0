pub mod vault_grpc;
pub mod tonic_service;

pub use vault_grpc::VaultGrpcService;
pub use tonic_service::VaultServiceTonic;
