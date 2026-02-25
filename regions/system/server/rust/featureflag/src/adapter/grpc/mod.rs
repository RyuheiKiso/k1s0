pub mod featureflag_grpc;
pub mod tonic_service;

pub use featureflag_grpc::FeatureFlagGrpcService;
pub use tonic_service::FeatureFlagServiceTonic;
