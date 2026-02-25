pub mod notification_grpc;
pub mod tonic_service;

pub use notification_grpc::NotificationGrpcService;
pub use tonic_service::NotificationServiceTonic;
