// AI Gateway tonic gRPCサービス実装。
// protoから生成されたトレイトを実装し、gRPCリクエストを処理する。

use std::sync::Arc;

use tonic::{Request, Response, Status};

use super::ai_grpc::{
    AiGatewayGrpcService, GrpcCompleteRequest, GrpcEmbedRequest, GrpcError, GrpcGetUsageRequest,
    GrpcMessage,
};

/// GrpcErrorからtonic::Statusへの変換。
impl From<GrpcError> for Status {
    fn from(e: GrpcError) -> Self {
        match e {
            GrpcError::InvalidArgument(msg) => Status::invalid_argument(msg),
            GrpcError::NotFound(msg) => Status::not_found(msg),
            GrpcError::Internal(msg) => Status::internal(msg),
        }
    }
}

/// AI Gateway tonicサービス。
/// proto生成のサービストレイトを実装するラッパー。
pub struct AiGatewayServiceTonic {
    inner: Arc<AiGatewayGrpcService>,
}

impl AiGatewayServiceTonic {
    /// 新しいtonicサービスを生成する。
    pub fn new(inner: Arc<AiGatewayGrpcService>) -> Self {
        Self { inner }
    }

    /// 内部サービスへの参照を取得する。
    pub fn inner(&self) -> &Arc<AiGatewayGrpcService> {
        &self.inner
    }
}
