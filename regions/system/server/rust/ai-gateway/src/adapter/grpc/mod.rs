pub mod ai_grpc;
pub mod tonic_service;

pub use ai_grpc::AiGatewayGrpcService;
pub use tonic_service::AiGatewayServiceTonic;
