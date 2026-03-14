// gRPCモジュール
// Tonicサービス実装とビジネスロジックを公開する

pub mod agent_grpc;
pub mod tonic_service;

pub use agent_grpc::AiAgentGrpcService;
pub use tonic_service::AiAgentServiceTonic;
