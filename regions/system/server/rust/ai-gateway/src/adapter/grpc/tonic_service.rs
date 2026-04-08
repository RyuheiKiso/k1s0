// AI Gateway tonic gRPCサービス実装。
// protoから生成されたトレイトを実装し、gRPCリクエストを処理する。

use std::sync::Arc;

use tonic::Status;

use super::ai_grpc::{AiGatewayGrpcService, GrpcError};

/// `GrpcErrorからtonic::Statusへの変換`。
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
#[allow(dead_code)]
pub struct AiGatewayServiceTonic {
    inner: Arc<AiGatewayGrpcService>,
}

#[allow(dead_code)]
impl AiGatewayServiceTonic {
    /// 新しいtonicサービスを生成する。
    #[must_use] 
    pub fn new(inner: Arc<AiGatewayGrpcService>) -> Self {
        Self { inner }
    }

    /// 内部サービスへの参照を取得する。
    #[must_use] 
    pub fn inner(&self) -> &Arc<AiGatewayGrpcService> {
        &self.inner
    }
}
